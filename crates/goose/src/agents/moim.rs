use crate::agents::extension_manager::ExtensionManager;
use crate::conversation::message::MessageContent;
use crate::conversation::{effective_role, fix_conversation, Conversation};
use std::path::PathBuf;

const MIN_CONTEXT_FOR_MOIM: usize = 32_000;
const TURN_CONTEXT_TAG: &str = "turn-context";

thread_local! {
    pub static SKIP: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MoimStatus {
    pub turns_taken: Option<u32>,
    pub max_turns: Option<u32>,
}

#[derive(Debug, Clone, Default)]
struct TurnContext {
    working_dir: PathBuf,
    total_tokens: Option<i32>,
    context_limit: Option<usize>,
    compaction_threshold: Option<f64>,
    turns_taken: Option<u32>,
    max_turns: Option<u32>,
}

pub async fn inject_moim(
    session_id: &str,
    conversation: Conversation,
    extension_manager: &ExtensionManager,
    status: MoimStatus,
) -> Conversation {
    if SKIP.with(|f| f.get()) {
        return conversation;
    }

    let session = extension_manager
        .get_context()
        .session_manager
        .get_session(session_id, false)
        .await
        .ok();
    let provider_context_limit = extension_manager
        .get_provider()
        .try_lock()
        .ok()
        .and_then(|provider| {
            provider
                .as_ref()
                .map(|provider| provider.get_model_config().context_limit())
        });
    let session_context_limit = session
        .as_ref()
        .and_then(|session| session.model_config.as_ref().map(|config| config.context_limit()));
    let context = TurnContext {
        working_dir: session
            .as_ref()
            .map(|session| session.working_dir.clone())
            .unwrap_or_else(|| PathBuf::from(".")),
        total_tokens: session.as_ref().and_then(|session| session.total_tokens),
        context_limit: provider_context_limit.or(session_context_limit),
        compaction_threshold: Some(
            crate::config::Config::global()
                .get_param::<f64>("GOOSE_AUTO_COMPACT_THRESHOLD")
                .unwrap_or(crate::context_mgmt::DEFAULT_COMPACTION_THRESHOLD),
        ),
        turns_taken: status.turns_taken,
        max_turns: status.max_turns,
    };
    if should_skip_moim(&context) {
        return conversation;
    }

    let extension_parts = extension_manager.collect_moim_parts(session_id).await;
    let moim = compose_moim(&context, extension_parts);

    let mut messages = conversation.messages().clone();
    let Some(idx) = messages
        .iter()
        .rposition(|m| effective_role(m) == "user")
    else {
        return conversation;
    };
    let insert_idx = messages[idx]
        .content
        .iter()
        .take_while(|content| matches!(content, MessageContent::ToolResponse(_)))
        .count();
    messages[idx]
        .content
        .insert(insert_idx, MessageContent::text(moim));

    let (fixed, issues) = fix_conversation(Conversation::new_unvalidated(messages));

    let has_unexpected_issues = issues.iter().any(|issue| {
        !issue.contains("Merged consecutive user messages")
            && !issue.contains("Merged consecutive assistant messages")
            && !issue.contains("Added placeholder to empty tool result")
            && !issue.contains("Trimmed trailing whitespace from assistant message")
            && !issue.contains("Removed trailing assistant message")
            && !issue.contains("Merged text content")
    });

    if has_unexpected_issues {
        tracing::warn!("MOIM injection caused unexpected issues: {:?}", issues);
        return conversation;
    }

    fixed
}

fn should_skip_moim(context: &TurnContext) -> bool {
    context
        .context_limit
        .is_some_and(|limit| limit < MIN_CONTEXT_FOR_MOIM)
}

fn compose_moim(context: &TurnContext, extension_parts: Vec<String>) -> String {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:00");
    let mut lines = vec![
        open_tag(TURN_CONTEXT_TAG),
        tag("current-time", &timestamp.to_string()),
        tag("working-directory", &context.working_dir.display().to_string()),
    ];

    if let Some(value) = compaction_remaining_line(context) {
        lines.push(tag("compaction", &value));
    }
    if let Some(value) = turn_budget_line(context) {
        lines.push(tag("turn-budget", &value));
    }

    for part in extension_parts {
        if !part.trim().is_empty() {
            lines.push(String::new());
            lines.push(part);
        }
    }

    lines.push(close_tag(TURN_CONTEXT_TAG));
    lines.join("\n")
}

fn open_tag(name: &str) -> String {
    format!("<{name}>")
}

fn close_tag(name: &str) -> String {
    format!("</{name}>")
}

fn tag(name: &str, value: &str) -> String {
    format!("<{name}>{}</{name}>", escape_xml_text(value))
}

fn escape_xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn compaction_remaining_line(context: &TurnContext) -> Option<String> {
    let total_tokens = context.total_tokens?;
    let context_limit = context.context_limit?;
    let threshold = context.compaction_threshold?;

    if total_tokens <= 0 || context_limit == 0 || threshold <= 0.0 || threshold >= 1.0 {
        return None;
    }

    let compaction_at = (context_limit as f64 * threshold) as i32;
    if compaction_at <= 0 || (total_tokens as f64 / compaction_at as f64) < 0.5 {
        return None;
    }

    Some(format!(
        "~{}k tokens remaining",
        compaction_at.saturating_sub(total_tokens) / 1000
    ))
}

fn turn_budget_line(context: &TurnContext) -> Option<String> {
    let (turns_taken, max_turns) = (context.turns_taken?, context.max_turns?);
    if max_turns == 0 || turns_taken.saturating_mul(2) < max_turns {
        return None;
    }

    Some(format!("{turns_taken}/{max_turns} used"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::message::Message;
    use rmcp::model::CallToolRequestParams;

    fn status() -> MoimStatus {
        MoimStatus::default()
    }

    fn turn_context() -> TurnContext {
        TurnContext {
            working_dir: PathBuf::from("/test/dir"),
            ..Default::default()
        }
    }

    fn text_at(message: &crate::conversation::message::Message, index: usize) -> &str {
        message.content[index].as_text().unwrap()
    }

    fn is_moim(content: &MessageContent) -> bool {
        content
            .as_text()
            .is_some_and(|text| text.starts_with(&format!("<{}>\n", TURN_CONTEXT_TAG)))
    }

    #[tokio::test]
    async fn test_moim_prepended_to_latest_user_message() {
        let temp_dir = tempfile::tempdir().unwrap();
        let em = ExtensionManager::new_without_provider(temp_dir.path().to_path_buf());
        let session = em
            .get_context()
            .session_manager
            .create_session(
                PathBuf::from("/test/dir"),
                "test".to_string(),
                crate::session::SessionType::User,
                crate::config::GooseMode::Auto,
            )
            .await
            .unwrap();

        let conv = Conversation::new_unvalidated(vec![
            Message::user().with_text("Hello"),
            Message::assistant().with_text("Hi"),
            Message::user().with_text("Bye"),
        ]);
        let result = inject_moim(&session.id, conv, &em, status()).await;
        let msgs = result.messages();

        assert_eq!(msgs.len(), 3);
        assert_eq!(text_at(&msgs[0], 0), "Hello");
        assert_eq!(text_at(&msgs[1], 0), "Hi");
        assert!(is_moim(&msgs[2].content[0]));
        assert_eq!(text_at(&msgs[2], 1), "Bye");
    }

    #[tokio::test]
    async fn test_moim_injection_no_assistant() {
        let temp_dir = tempfile::tempdir().unwrap();
        let em = ExtensionManager::new_without_provider(temp_dir.path().to_path_buf());
        let session = em
            .get_context()
            .session_manager
            .create_session(
                PathBuf::from("/test/dir"),
                "test".to_string(),
                crate::session::SessionType::User,
                crate::config::GooseMode::Auto,
            )
            .await
            .unwrap();

        let conv = Conversation::new_unvalidated(vec![Message::user().with_text("Hello")]);
        let result = inject_moim(&session.id, conv, &em, status()).await;

        assert_eq!(result.messages().len(), 1);
        assert!(is_moim(&result.messages()[0].content[0]));
        assert_eq!(text_at(&result.messages()[0], 1), "Hello");
    }

    #[tokio::test]
    async fn test_moim_with_tool_calls() {
        let temp_dir = tempfile::tempdir().unwrap();
        let em = ExtensionManager::new_without_provider(temp_dir.path().to_path_buf());
        let session = em
            .get_context()
            .session_manager
            .create_session(
                PathBuf::from("/test/dir"),
                "test".to_string(),
                crate::session::SessionType::User,
                crate::config::GooseMode::Auto,
            )
            .await
            .unwrap();

        let conv = Conversation::new_unvalidated(vec![
            Message::user().with_text("Search for something"),
            Message::assistant()
                .with_text("I'll search for you")
                .with_tool_request("search_1", Ok(CallToolRequestParams::new("search"))),
            Message::user()
                .with_tool_response("search_1", Ok(rmcp::model::CallToolResult::success(vec![]))),
        ]);

        let result = inject_moim(&session.id, conv, &em, status()).await;
        let msgs = result.messages();

        assert_eq!(msgs.len(), 3);
        assert!(is_moim(&msgs[0].content[0]));
        assert_eq!(text_at(&msgs[0], 1), "Search for something");
        assert!(matches!(
            &msgs[2].content[0],
            MessageContent::ToolResponse(_)
        ));
        assert_eq!(msgs[2].content.len(), 1);
    }

    #[test]
    fn test_compaction_remaining_line_threshold() {
        let base = TurnContext {
            working_dir: PathBuf::from("/tmp"),
            context_limit: Some(100_000),
            compaction_threshold: Some(0.8),
            ..Default::default()
        };

        assert_eq!(
            compaction_remaining_line(&TurnContext {
                total_tokens: Some(10_000),
                ..base.clone()
            }),
            None
        );
        assert_eq!(
            compaction_remaining_line(&TurnContext {
                total_tokens: Some(50_000),
                ..base.clone()
            }),
            Some("~30k tokens remaining".to_string())
        );
        assert_eq!(
            compaction_remaining_line(&TurnContext {
                total_tokens: Some(50_000),
                compaction_threshold: Some(0.0),
                ..base.clone()
            }),
            None
        );
        assert_eq!(
            compaction_remaining_line(&TurnContext {
                total_tokens: Some(50_000),
                compaction_threshold: Some(1.0),
                ..base
            }),
            None
        );
    }

    #[test]
    fn test_turn_budget_line_threshold() {
        assert_eq!(
            turn_budget_line(&TurnContext {
                turns_taken: Some(49),
                max_turns: Some(100),
                ..turn_context()
            }),
            None
        );
        assert_eq!(
            turn_budget_line(&TurnContext {
                turns_taken: Some(50),
                max_turns: Some(100),
                ..turn_context()
            }),
            Some("50/100 used".to_string())
        );
        assert_eq!(
            turn_budget_line(&TurnContext {
                turns_taken: Some(1),
                max_turns: Some(0),
                ..turn_context()
            }),
            None
        );
    }
}

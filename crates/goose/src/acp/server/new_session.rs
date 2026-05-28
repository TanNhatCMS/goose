use crate::acp::server::{meta_string, sid_short, validate_absolute_cwd, ResultExt};
use crate::config::{Config, GooseMode};
use crate::session::SessionType;

use super::GooseAcpAgent;
use agent_client_protocol::schema::{NewSessionRequest, NewSessionResponse};
use agent_client_protocol::{Client, ConnectionTo};
use tracing::debug;

impl GooseAcpAgent {
    #[allow(dead_code)]
    pub(super) async fn handle_new_session(
        &self,
        cx: &ConnectionTo<Client>,
        args: NewSessionRequest,
    ) -> Result<NewSessionResponse, agent_client_protocol::Error> {
        debug!(?args, "new session request");
        let t_start = std::time::Instant::now();
        validate_absolute_cwd(&args.cwd)?;
        let project_id = meta_string(args.meta.as_ref(), "projectId");
        let session_type = match meta_string(args.meta.as_ref(), "client") {
            Some(_) => SessionType::User,
            None => SessionType::Acp,
        };
        let config = Config::global();
        let current_mode: GooseMode = config.get_goose_mode().unwrap_or_default();
        let t0 = std::time::Instant::now();
        let goose_session = self
            .session_manager
            .create_session(
                args.cwd.clone(),
                "New Chat".to_string(),
                session_type,
                current_mode,
            )
            .await
            .internal_err_ctx("Failed to create session")?;
        let mut builder = self.session_manager.update(&goose_session.id);
        if let Some(pid) = project_id {
            builder = builder.project_id(Some(pid));
        }
        builder
            .apply()
            .await
            .internal_err_ctx("Failed to update session")?;

        let goose_session = self
            .session_manager
            .get_session(&goose_session.id, false)
            .await
            .internal_err_ctx("Failed to reload session")?;
        let session_id_str = goose_session.id.clone();
        let sid = sid_short(&session_id_str);
        let init_state = self.prepare_session_init_state(&goose_session).await?;
        debug!(target: "perf", sid = %sid, ms = t0.elapsed().as_millis() as u64, "perf: new_session create_session");

        todo!("move new_session handling from server.rs incrementally")
    }
}

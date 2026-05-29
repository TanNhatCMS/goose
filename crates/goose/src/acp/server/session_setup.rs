use super::*;
use crate::session::{ExtensionData, ExtensionState};

pub(super) fn resolve_default_provider_model_config(
    config: &Config,
) -> Result<(String, crate::model::ModelConfig), agent_client_protocol::Error> {
    let resolved_provider = config.get_goose_provider().map_err(|error| {
        agent_client_protocol::Error::internal_error()
            .data(format!("Failed to resolve provider: {}", error))
    })?;
    let resolved_model = config.get_goose_model().map_err(|error| {
        agent_client_protocol::Error::internal_error()
            .data(format!("Failed to resolve model: {}", error))
    })?;
    let resolved_model_config = crate::model::ModelConfig::new(&resolved_model)
        .map(|model_config| model_config.with_canonical_limits(&resolved_provider))
        .map_err(|error| {
            agent_client_protocol::Error::internal_error()
                .data(format!("Failed to resolve model: {}", error))
        })?;
    Ok((resolved_provider, resolved_model_config))
}

pub(super) fn build_enabled_extensions_data(
    agent: &GooseAcpAgent,
    config: &Config,
    session: &Session,
    mcp_servers: Vec<McpServer>,
) -> Result<ExtensionData, agent_client_protocol::Error> {
    let extensions = agent.initial_session_extensions(config, mcp_servers)?;
    let mut extension_data = session.extension_data.clone();
    EnabledExtensionsState::new(extensions)
        .to_extension_data(&mut extension_data)
        .internal_err_ctx("Failed to initialize session extensions")?;
    Ok(extension_data)
}

pub(super) async fn prepare_session_for_activation(
    agent: &GooseAcpAgent,
    mut session: Session,
    cwd: std::path::PathBuf,
    mcp_servers: Vec<McpServer>,
    include_messages_on_reload: bool,
) -> Result<Session, agent_client_protocol::Error> {
    let config = Config::global();
    let mut builder = agent.session_manager.update(&session.id);
    let mut session_needs_update = false;

    if cwd != session.working_dir {
        builder = builder.working_dir(cwd);
        session_needs_update = true;
    }

    if session.provider_name.is_none() || session.model_config.is_none() {
        let (resolved_provider, resolved_model_config) =
            resolve_default_provider_model_config(config)?;
        builder = builder
            .provider_name(resolved_provider)
            .model_config(resolved_model_config);
        session_needs_update = true;
    }

    if !mcp_servers.is_empty()
        || EnabledExtensionsState::from_extension_data(&session.extension_data).is_none()
    {
        let extension_data = build_enabled_extensions_data(agent, config, &session, mcp_servers)?;
        builder = builder.extension_data(extension_data);
        session_needs_update = true;
    }

    if session_needs_update {
        let session_id = session.id.clone();
        builder
            .apply()
            .await
            .internal_err_ctx("Failed to update session")?;
        session = agent
            .session_manager
            .get_session(&session_id, include_messages_on_reload)
            .await
            .internal_err_ctx("Failed to reload session")?;
    }

    Ok(session)
}

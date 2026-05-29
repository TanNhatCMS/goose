use super::*;

impl GooseAcpAgent {
    #[allow(dead_code)]
    pub(super) async fn handle_fork_session(
        &self,
        cx: &ConnectionTo<Client>,
        args: ForkSessionRequest,
    ) -> Result<ForkSessionResponse, agent_client_protocol::Error> {
        validate_absolute_cwd(&args.cwd)?;
        let source_session_id = &*args.session_id.0;

        let source = self
            .session_manager
            .get_session(source_session_id, false)
            .await
            .internal_err()?;
        let fork_name = if source.name.trim().is_empty() {
            "(copy)".to_string()
        } else {
            format!("{} (copy)", source.name)
        };

        let new_session = self
            .session_manager
            .copy_session(source_session_id, fork_name)
            .await
            .internal_err()?;
        let new_session_id = new_session.id.clone();

        let goose_session = self
            .session_manager
            .get_session(&new_session_id, false)
            .await
            .internal_err()?;

        let goose_session = super::session_setup::prepare_session_for_activation(
            self,
            goose_session,
            args.cwd.clone(),
            args.mcp_servers,
            false,
        )
        .await?;

        let (_agent, _extension_results) = self
            .activate_acp_session(cx, &goose_session, HashMap::new())
            .await?;

        let acp_session_id = SessionId::new(new_session_id.clone());
        let meta = session_meta(&new_session);

        let mode_state = build_mode_state(goose_session.goose_mode)?;
        let (model_state, config_options) = self
            .build_eager_session_config(&mode_state, &goose_session)
            .await;

        let mut response = ForkSessionResponse::new(acp_session_id.clone())
            .modes(mode_state)
            .meta(meta);

        if let Some(ms) = model_state {
            response = response.models(ms);
        }
        if let Some(co) = config_options {
            response = response.config_options(co);
        }
        Self::send_available_commands_update(cx, &acp_session_id, &args.cwd)?;
        Ok(response)
    }
}

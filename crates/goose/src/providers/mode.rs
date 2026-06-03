use async_trait::async_trait;

#[cfg(feature = "aws-providers")]
use super::bedrock::BedrockProvider;
#[cfg(feature = "local-inference")]
use super::local_inference::LocalInferenceProvider;
#[cfg(feature = "aws-providers")]
use super::sagemaker_tgi::SageMakerTgiProvider;
use super::{
    anthropic::AnthropicProvider, chatgpt_codex::ChatGptCodexProvider,
    cursor_agent::CursorAgentProvider, databricks::DatabricksProvider,
    databricks_v2::DatabricksV2Provider, gcpvertexai::GcpVertexAIProvider,
    gemini_cli::GeminiCliProvider, gemini_oauth::GeminiOAuthProvider,
    githubcopilot::GithubCopilotProvider, google::GoogleProvider, huggingface::HuggingFaceProvider,
    kimicode::KimiCodeProvider, litellm::LiteLLMProvider, nanogpt::NanoGptProvider,
    ollama::OllamaProvider, openai::OpenAiProvider, openai_compatible::OpenAiCompatibleProvider,
    openrouter::OpenRouterProvider, snowflake::SnowflakeProvider, testprovider::TestProvider,
    tetrate::TetrateProvider, xai_oauth::XaiOAuthProvider,
};
use crate::config::GooseMode;
use crate::providers::base::Provider;
use crate::providers::errors::ProviderError;

#[async_trait]
pub trait GooseProvider: Provider {
    async fn update_mode(&self, _session_id: &str, _mode: GooseMode) -> Result<(), ProviderError> {
        Ok(())
    }
}

macro_rules! impl_default_goose_provider {
    ($($provider:ty),* $(,)?) => {
        $(
            #[async_trait]
            impl GooseProvider for $provider {}
        )*
    };
}

impl_default_goose_provider!(
    AnthropicProvider,
    ChatGptCodexProvider,
    CursorAgentProvider,
    DatabricksProvider,
    DatabricksV2Provider,
    GcpVertexAIProvider,
    GeminiCliProvider,
    GeminiOAuthProvider,
    GithubCopilotProvider,
    GoogleProvider,
    HuggingFaceProvider,
    KimiCodeProvider,
    LiteLLMProvider,
    NanoGptProvider,
    OllamaProvider,
    OpenAiProvider,
    OpenAiCompatibleProvider,
    OpenRouterProvider,
    SnowflakeProvider,
    TestProvider,
    TetrateProvider,
    XaiOAuthProvider,
);

#[cfg(feature = "aws-providers")]
impl_default_goose_provider!(BedrockProvider, SageMakerTgiProvider);

#[cfg(feature = "local-inference")]
impl_default_goose_provider!(LocalInferenceProvider);

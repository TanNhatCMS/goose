## Goal

Extract `crates/goose/src/providers` into a new workspace crate, `goose-providers`, with **no dependency on `goose`**. The new crate should expose the provider API centered on `Provider` from `providers/base.rs`, plus the types required to call it. Any current dependencies from providers back into `goose` must either:

1. Move into `goose-providers` because they are part of the provider API/model.
2. Be replaced by injected interfaces/runtime services.
3. Stay in `goose`, with `goose` adapting to `goose-providers`.

The largest required redesign is configuration: provider code should no longer read/write `crate::config::Config::global()` directly. Config must be injected.

---

# Summary of Current Coupling

I inspected `crates/goose/src/providers`. The provider module currently references these `goose` modules:

| Goose dependency          | Approx. role in providers                                                                                    | Proposed treatment                                                                                                               |
|---------------------------|--------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------|
| `crate::config`           | Config read/write, secrets, paths, search paths, `ExtensionConfig`, `GooseMode`, declarative provider config | Split: inject config/paths/search; move/provider-owned DTOs for mode and declarative config; convert extension config in `goose` |
| `crate::conversation`     | `Message`, `MessageContent`, `Conversation`, tool request/response content                                   | Move API model types into `goose-providers`; `goose` reexports/uses them                                                         |
| `crate::model`            | `ModelConfig`, `ThinkingEffort`, canonical model lookup, env-derived model settings                          | Move `ModelConfig`/`ThinkingEffort`; remove hidden config reads; inject model defaults                                           |
| `crate::providers`        | Internal provider modules                                                                                    | Becomes local module references inside `goose-providers`                                                                         |
| `crate::session_context`  | `SESSION_ID_HEADER` for HTTP providers                                                                       | Move constant to `goose-providers`                                                                                               |
| `crate::utils`            | `safe_truncate`, `bytes_to_hex`, unicode sanitization                                                        | Move small pure helpers or inline/replace                                                                                        |
| `crate::subprocess`       | Process setup for CLI/ACP providers                                                                          | Inject process configuration/command spawning service, or move only pure process helper                                          |
| `crate::mcp_utils`        | `ToolResult`, resource text extraction                                                                       | Move provider-facing tool/resource helpers                                                                                       |
| `crate::acp`              | ACP subprocess/session providers                                                                             | Do not move wholesale initially; define injected ACP session/client factory                                                      |
| `crate::permission`       | Permission confirmation types exposed by `Provider`                                                          | Move provider-facing permission DTOs                                                                                             |
| `crate::agents`           | `ExtensionConfig`, `Envs`, test fixtures                                                                     | Do not move goose agent config; introduce provider-owned extension/MCP DTO and convert in `goose`                                |
| `crate::prompt_template`  | Session naming prompt and local inference prompt                                                             | Move prompt strings or inject prompt renderer                                                                                    |
| `crate::token_counter`    | Usage estimation fallback                                                                                    | Inject token counter/usage estimator or move pure estimator behind provider trait                                                |
| `crate::session`          | Provider inventory persistence                                                                               | Keep storage/service in `goose` or abstract storage                                                                              |
| `crate::instance_id`      | Databricks request ID                                                                                        | Inject instance id                                                                                                               |
| `crate::download_manager` | Local inference model downloads                                                                              | Inject download/model-cache service, or defer local inference extraction                                                         |
---

# Primary API Impact

`Provider` currently has this shape:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn get_name(&self) -> &str;

    async fn stream(
        &self,
        model_config: &ModelConfig,
        session_id: &str,
        system: &str,
        messages: &[Message],
        tools: &[Tool],
    ) -> Result<MessageStream, ProviderError>;

    async fn complete(...) -> Result<(Message, ProviderUsage), ProviderError>;
    async fn complete_fast(...) -> Result<(Message, ProviderUsage), ProviderError>;

    fn get_model_config(&self) -> ModelConfig;
    fn retry_config(&self) -> RetryConfig;
    async fn fetch_supported_models(&self) -> Result<Vec<String>, ProviderError>;
    async fn fetch_supported_model_info(&self) -> Result<Vec<ModelInfo>, ProviderError>;
    async fn generate_session_name(..., messages: &Conversation) -> Result<String, ProviderError>;
    async fn configure_oauth(&self) -> Result<(), ProviderError>;
    async fn refresh_credentials(&self) -> Result<(), ProviderError>;
    async fn update_mode(&self, _session_id: &str, _mode: GooseMode) -> Result<(), ProviderError>;
    async fn handle_permission_confirmation(
        &self,
        _request_id: &str,
        _confirmation: &PermissionConfirmation,
    ) -> bool;
}
```

Because `Provider` directly uses `ModelConfig`, `Message`, `Conversation`, `GooseMode`, and `PermissionConfirmation`, those types cannot remain only in `goose` if `goose-providers` is independent. We need to move or replace them.

## Types that should move to `goose-providers`

These are part of the provider-facing API and should move with the trait:

- `Provider`
- `ProviderDef`, though redesigned
- `ProviderError`
- `MessageStream`
- `ProviderUsage`
- `Usage`
- `RetryConfig`, retry helpers
- `ModelInfo`
- `ProviderMetadata`
- `ConfigKey`
- `ProviderType`
- `PermissionRouting`
- Provider-facing `Permission`, `PermissionConfirmation`, `PrincipalType`
- `ModelConfig`
- `ThinkingEffort`
- `Message`
- `MessageContent`
- Tool request/response structs
- `Conversation` or a lighter provider-facing conversation wrapper
- `ToolResult`
- Resource text extraction helpers needed to convert MCP content
- `DEFAULT_PROVIDER_TIMEOUT_SECS`
- canonical model metadata/registry helpers, because `Provider` defaults and `ModelConfig` use them

`goose` can then reexport these where needed to minimize downstream churn.

---

# Config Redesign

## Current state

Provider constructors and methods frequently call:

- `Config::global()`
- `get_param`
- `get_secret`
- `get_secrets`
- `set_param`
- `set_secret`
- `invalidate_secrets_cache`
- `Paths::config_dir`, `Paths::state_dir`, `Paths::in_config_dir`
- `SearchPaths::builder()...`

Examples:

- OpenAI reads `OPENAI_API_KEY`, host, base path, org/project, custom headers, timeout.
- Anthropic reads `ANTHROPIC_API_KEY`, host, custom headers.
- Databricks reads host/token/retry config and invalidates secret cache on auth retry.
- GitHub Copilot writes refreshed OAuth token back to config.
- Kimi writes `kimi_code_configured` marker.
- Ollama tests and setup write `OLLAMA_HOST`.
- ACP/CLI providers read current `GooseMode`, command config, and resolve binaries.
- OAuth providers write/read token cache files under goose config dirs.

This is the largest violation of the “no `goose` dependency” rule.

## Proposed injected config interface

Create object-safe provider config traits in `goose-providers`:

```rust
pub trait ProviderConfigStore: Send + Sync {
    fn get_param_value(&self, key: &str) -> Result<serde_json::Value, ProviderConfigError>;
    fn get_secret_value(&self, key: &str) -> Result<serde_json::Value, ProviderConfigError>;

    fn get_secret_group(
        &self,
        primary: &str,
        maybe_secret: &[&str],
    ) -> Result<std::collections::HashMap<String, String>, ProviderConfigError>;

    fn set_param_value(
        &self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), ProviderConfigError>;

    fn set_secret_value(
        &self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), ProviderConfigError>;

    fn delete_secret(&self, key: &str) -> Result<(), ProviderConfigError>;

    fn invalidate_secrets_cache(&self);
}
```

Then add generic extension helpers:

```rust
pub trait ProviderConfigExt {
    fn get_param<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<T, ProviderConfigError>;

    fn get_secret<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<T, ProviderConfigError>;

    fn set_param<T: serde::Serialize>(
        &self,
        key: &str,
        value: T,
    ) -> Result<(), ProviderConfigError>;

    fn set_secret<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), ProviderConfigError>;
}
```

`goose` implements `ProviderConfigStore` for an adapter around `crate::config::Config`.

### Error type

Add a provider-owned config error:

```rust
pub enum ProviderConfigError {
    NotFound(String),
    Deserialize(String),
    Storage(String),
    SecretStorage(String),
}
```

`goose` maps `crate::config::ConfigError` into this type.

This preserves important current behavior, such as distinguishing “not found” from “keyring failed”.

---

# Runtime Injection

Config alone is not enough. Several providers need filesystem paths, command resolution, process setup, prompt rendering, token counting, ACP sessions, and downloads.

Introduce a provider runtime object:

```rust
pub struct ProviderRuntime {
    pub config: Arc<dyn ProviderConfigStore>,
    pub paths: Arc<dyn ProviderPaths>,
    pub commands: Arc<dyn CommandResolver>,
    pub process: Arc<dyn ProcessConfigurator>,
    pub prompts: Arc<dyn PromptRenderer>,
    pub token_counter: Arc<dyn TokenCounter>,
    pub acp: Option<Arc<dyn AcpSessionFactory>>,
    pub downloads: Option<Arc<dyn DownloadManager>>,
    pub instance_id: Option<String>,
    pub mode: ProviderMode,
}
```

Some fields can be optional or split into smaller structs to avoid forcing simple HTTP providers to care about ACP/downloads.

## Provider construction

Replace `ProviderDef::from_env` with a context-based constructor:

```rust
pub struct ProviderInit {
    pub model: ModelConfig,
    pub extensions: Vec<ProviderExtensionConfig>,
    pub working_dir: Option<PathBuf>,
    pub runtime: Arc<ProviderRuntime>,
}

pub trait ProviderDef: Send + Sync {
    type Provider: Provider + 'static;

    fn metadata() -> ProviderMetadata;

    fn from_init(init: ProviderInit) -> BoxFuture<'static, anyhow::Result<Self::Provider>>;

    fn supports_inventory_refresh() -> bool {
        false
    }

    fn inventory_identity(
        runtime: &ProviderRuntime,
    ) -> anyhow::Result<InventoryIdentityInput> {
        ...
    }

    fn inventory_configured(runtime: &ProviderRuntime) -> bool {
        ...
    }
}
```

This renames the current misleading `from_env`: providers will no longer own global config/env lookup.

---

# Extension Config Redesign

Current constructors take `Vec<crate::config::ExtensionConfig>`, which is actually reexported from `crate::agents::ExtensionConfig`. That type is part of the goose agent configuration model and should not move wholesale into `goose-providers`.

Instead define a provider-owned DTO with only what providers need:

```rust
pub enum ProviderExtensionConfig {
    Stdio {
        name: String,
        cmd: String,
        args: Vec<String>,
        env: std::collections::HashMap<String, String>,
        timeout: Option<u64>,
        available_tools: Vec<String>,
    },
    StreamableHttp {
        name: String,
        uri: String,
        headers: std::collections::HashMap<String, String>,
        timeout: Option<u64>,
        available_tools: Vec<String>,
    },
}
```

Then `goose` converts from `goose::agents::ExtensionConfig` to `ProviderExtensionConfig` before constructing providers.

This isolates provider crates from goose agent config while still supporting Codex/Claude/ACP MCP configuration generation.

---

# Model Config Redesign

`ModelConfig` currently has hidden config reads:

- `GOOSE_CONTEXT_LIMIT`
- max tokens
- temperature
- toolshim
- toolshim model
- predefined models
- canonical model lookup

That means `ModelConfig::new()` is not pure and currently reaches into `crate::config` and `crate::providers`.

## Plan

Move `ModelConfig` and `ThinkingEffort` to `goose-providers`, but make `ModelConfig::new` pure:

```rust
impl ModelConfig {
    pub fn new(model_name: &str) -> Result<Self, ModelConfigError> {
        ...
    }
}
```

Add a resolver/builder that can use injected config:

```rust
pub struct ModelConfigResolver {
    runtime: Arc<ProviderRuntime>,
}

impl ModelConfigResolver {
    pub fn resolve(
        &self,
        provider_name: &str,
        model_name: &str,
    ) -> Result<ModelConfig, ModelConfigError> {
        // read GOOSE_CONTEXT_LIMIT, GOOSE_MAX_TOKENS, etc via runtime.config
        // then apply canonical limits
    }
}
```

`goose` should use this resolver when creating providers from active config or CLI/server inputs.

---

# Dependency Treatment by Module

## `crate::config`

### Current uses

- Direct config reads/writes.
- Secret reads/writes.
- Config cache invalidation.
- `ConfigError` matching.
- `Paths`.
- `SearchPaths`.
- `ExtensionConfig`.
- `GooseMode`.
- typed command config wrappers like `ClaudeCodeCommand`, `CodexCommand`, `GeminiCliCommand`, `CursorAgentCommand`.
- declarative provider schemas.

### Plan

- Replace direct `Config::global()` calls with injected `ProviderConfigStore`.
- Replace `Paths` with injected `ProviderPaths`.
- Replace `SearchPaths` with injected `CommandResolver`.
- Move/rename `GooseMode` to provider-owned `ProviderMode`.
- Do not move goose `ExtensionConfig`; convert to provider-owned `ProviderExtensionConfig`.
- Replace typed config wrapper dependencies with constants/defaults in providers, or use generic config keys through `ProviderConfigStore`.
- Move declarative provider schema pieces that are provider-specific into `goose-providers`; keep file CRUD and UI/server management in `goose` unless abstracted.

---

## `crate::conversation`

### Current uses

Provider APIs and all format converters use:

- `Message`
- `MessageContent`
- `ToolRequest`
- `ToolResponse`
- `Conversation`
- text/image/thinking/tool helper methods

### Plan

Move provider-facing conversation/message model into `goose-providers`.

`goose` then imports/reexports:

```rust
pub use goose_providers::conversation;
pub use goose_providers::conversation::message::{Message, MessageContent};
```

This avoids converting every request/response between goose and provider-specific message types.

---

## `crate::model`

### Current uses

- `ModelConfig`
- `ThinkingEffort`
- canonical model limits
- reasoning model detection
- model suffix normalization

### Plan

Move `ModelConfig` and `ThinkingEffort` to `goose-providers`.

Remove direct config access from `ModelConfig`.

Move canonical registry/data to `goose-providers`, because `ModelConfig`, `ModelInfo`, and provider metadata all rely on it.

---

## `crate::providers`

### Current uses

Most references are internal cross-provider/module imports.

### Plan

After extraction these become local `crate::...` or `super::...` references in `goose-providers`.

Specific notes:

- `errors`, `retry`, `api_client`, `http_status`, `formats`, `canonical`, `catalog`, most `utils` should move.
- `provider_registry` should move but be redesigned around injected runtime.
- `init` should split:
  - provider registration/bootstrap can live in `goose-providers`;
  - global singleton initialization with goose config/custom providers may stay in `goose`.
- `inventory` should split:
  - provider inventory DTOs/identity helpers move;
  - refresh service that depends on `SessionStorage` stays in `goose` or gets storage injection.

---

## `crate::session_context`

### Current uses

`SESSION_ID_HEADER` is inserted into HTTP requests by several providers.

### Plan

Move this constant to `goose-providers`, e.g.:

```rust
pub const SESSION_ID_HEADER: &str = "x-goose-session-id";
```

It is provider transport behavior, not core goose logic.

---

## `crate::utils`

### Current uses

- `safe_truncate`
- `bytes_to_hex`
- unicode tag sanitization

### Plan

Move these small pure helpers into `goose-providers::utils`, or inline them in relevant modules.

---

## `crate::subprocess`

### Current uses

- `configure_subprocess`
- `SubprocessExt`
- CLI providers: Claude Code, Codex, Gemini CLI, Cursor Agent
- auth helpers like Azure auth

### Plan

Do not make providers call goose subprocess helpers directly.

Introduce:

```rust
pub trait ProcessConfigurator: Send + Sync {
    fn configure(&self, command: &mut tokio::process::Command);
}
```

`goose` implements this with existing `crate::subprocess::configure_subprocess`.

For `SubprocessExt`, either:

- move it if it is generic/pure, or
- replace usage with a provider-local helper.

---

## `crate::mcp_utils`

### Current uses

- `ToolResult`
- `extract_text_from_resource`

These are used by message conversion and formatters.

### Plan

Move provider-facing MCP utility helpers to `goose-providers`.

Keep using `rmcp` as an external dependency of `goose-providers`.

---

## `crate::acp`

### Current uses

ACP providers (`amp_acp`, `claude_acp`, `codex_acp`, `copilot_acp`, `pi_acp`) call goose ACP session/client code.

### Plan

Do not move `crate::acp` wholesale as an implementation detail.

Add provider-facing ACP abstractions:

```rust
#[async_trait]
pub trait AcpSessionFactory: Send + Sync {
    async fn start_session(
        &self,
        config: AcpProviderLaunchConfig,
    ) -> Result<Box<dyn AcpSession>, ProviderError>;
}
```

ACP provider implementations in `goose-providers` depend only on this trait.

`goose` implements the factory using current `crate::acp`.

If this abstraction is too large for the first extraction, keep ACP providers in `goose` temporarily as adapters implementing `goose_providers::Provider`, then migrate them in a later phase.

---

## `crate::permission`

### Current uses

`Provider` exposes `PermissionConfirmation` in `handle_permission_confirmation`, and Claude Code uses permissions internally.

### Plan

Move provider-facing permission DTOs to `goose-providers`:

- `Permission`
- `PrincipalType`
- `PermissionConfirmation`

`goose` can reexport or convert if its broader permission system needs different types later.

---

## `crate::agents`

### Current uses

- `ExtensionConfig`
- `Envs`
- one test import of developer `ShellParams`

### Plan

Do not move goose agent config.

Define provider-owned extension DTOs as described above.

Replace tests that import goose platform-specific types with raw JSON fixtures or provider-local test structs.

---

## `crate::prompt_template`

### Current uses

- `Provider::generate_session_name` renders `session_name.md`.
- local inference renders `tiny_model_system.md`.

### Plan

Preferred: move session naming out of the `Provider` trait into goose orchestration. Providers should only complete prompts; goose can build the session-name prompt.

Alternative if preserving `Provider::generate_session_name`: inject `PromptRenderer`.

For local inference, either move the tiny prompt content into `goose-providers`, or inject prompt content through runtime.

---

## `crate::token_counter`

### Current uses

`ProviderUsage::ensure_tokens` calls `providers::usage_estimator`, which calls `crate::token_counter::create_token_counter`.

### Plan

Remove direct dependency from `ProviderUsage`.

Options:

1. Move token counting into `goose-providers` if it is sufficiently provider-generic.
2. Better: inject a `TokenCounter`/`UsageEstimator` trait.

Example:

```rust
#[async_trait]
pub trait UsageEstimator: Send + Sync {
    async fn ensure_usage_tokens(
        &self,
        usage: &mut ProviderUsage,
        system_prompt: &str,
        request_messages: &[Message],
        response: &Message,
        tools: &[Tool],
    ) -> Result<(), ProviderError>;
}
```

---

## `crate::session`

### Current uses

Provider inventory service references `SessionStorage`.

### Plan

Keep inventory persistence/refresh orchestration in `goose`, or introduce a provider-owned storage trait.

Move only provider inventory identity/configuration DTOs into `goose-providers`.

---

## `crate::instance_id`

### Current uses

Databricks optionally includes a client request ID based on goose instance id.

### Plan

Inject `instance_id: Option<String>` into `ProviderRuntime`.

Databricks reads `runtime.instance_id` instead of calling `get_instance_id()`.

---

## `crate::download_manager`

### Current uses

Local inference model registry/downloads.

### Plan

Local inference is likely the hardest provider family to extract because it depends on model registry paths and downloads.

Options:

1. Add `DownloadManager` and `LocalModelStore` traits to `goose-providers`, implemented by `goose`.
2. Defer local inference extraction behind a compatibility layer until HTTP/subprocess providers are migrated.

I recommend deferring local inference until after the main provider API extraction.

---

# Provider Families and Migration Order

## Phase 1: Create `goose-providers` API crate

Create `crates/goose-providers` with:

- `base`
- `errors`
- `retry`
- `model`
- `conversation`
- `message`
- `mcp_utils`
- provider-facing permission types
- provider runtime/config traits
- canonical model registry/data
- provider metadata types

Update `goose` to depend on `goose-providers` and reexport moved types.

Goal: compile with minimal provider implementation movement.

---

## Phase 2: Remove hidden config reads from shared types

Refactor:

- `ModelConfig::new`
- `ModelConfig::new_with_context_env`
- `with_canonical_limits`
- `ProviderDef::inventory_identity`
- `ProviderDef::inventory_configured`
- `ProviderUsage::ensure_tokens`
- `Provider::generate_session_name`

Any default trait method in `goose-providers` must not call into goose.

---

## Phase 3: Introduce goose runtime adapter

In `goose`, add an adapter layer:

```rust
pub struct GooseProviderRuntime {
    config: &'static goose::config::Config,
    ...
}
```

Implement:

- `ProviderConfigStore`
- `ProviderPaths`
- `CommandResolver`
- `ProcessConfigurator`
- `PromptRenderer`
- `TokenCounter`/`UsageEstimator`
- `AcpSessionFactory`
- `DownloadManager`, later

This gives provider constructors everything they need without importing `goose`.

---

## Phase 4: Move low-risk pure modules

Move modules with little/no goose coupling first:

- `errors.rs`
- `retry.rs`
- `http_status.rs`
- most `utils.rs`
- `api_client.rs`, after TLS config uses injected config
- `formats/*`
- `canonical/*`
- `catalog.rs`, if no runtime dependency remains

This reduces the import surface before moving provider implementations.

---

## Phase 5: Move HTTP providers

Start with providers that only need config + HTTP client:

- `openai`
- `anthropic`
- `google`
- `openrouter`
- `ollama`
- `litellm`
- `nanogpt`
- `tetrate`
- `xai`
- `avian`
- `azure`
- `snowflake`
- `huggingface`
- `gcpvertexai`
- `bedrock` / `sagemaker_tgi` behind feature flags
- `databricks` / `databricks_v2`

For each:

- Replace `Config::global()` with `init.runtime.config`.
- Store needed runtime/config handles in the provider struct only when used after construction.
- Replace direct cache invalidation with `runtime.config.invalidate_secrets_cache()`.
- Replace `SESSION_ID_HEADER` import with moved constant.
- Replace `Paths` with `runtime.paths`.

---

## Phase 6: Move OAuth/token-cache providers

Providers:

- `githubcopilot`
- `gemini_oauth`
- `xai_oauth`
- `kimicode`
- `huggingface_auth`
- `oauth`
- `oauth_device_flow`
- `databricks_auth`

Main work:

- Replace file paths with injected `ProviderPaths`.
- Replace secret writes with `ProviderConfigStore::set_secret_value`.
- Replace marker writes with `set_param_value`.
- Ensure config write capability is explicitly part of runtime.

---

## Phase 7: Move CLI/subprocess providers

Providers:

- `claude_code`
- `codex`
- `gemini_cli`
- `cursor_agent`
- `chatgpt_codex`

Main work:

- Replace `SearchPaths` with `CommandResolver`.
- Replace `configure_subprocess` with `ProcessConfigurator`.
- Replace `ExtensionConfig` with `ProviderExtensionConfig`.
- Replace `GooseMode` with `ProviderMode`.
- Move permission DTOs.
- Move or inject prompt/session-name helpers.

---

## Phase 8: Move ACP providers

Providers:

- `amp_acp`
- `claude_acp`
- `codex_acp`
- `copilot_acp`
- `pi_acp`

Main work:

- Add `AcpSessionFactory`.
- Convert extensions to provider-owned MCP server configs.
- Replace command resolution/mode/config calls with runtime services.

If needed, keep these as temporary wrappers in `goose` while the rest of `goose-providers` lands.

---

## Phase 9: Local inference

Provider:

- `local_inference`
- `local_inference/*`

Main work:

- Move local inference model/settings DTOs.
- Abstract model downloads/cache.
- Replace `Paths`.
- Replace prompt templates.
- Decide whether heavy optional dependencies live in `goose-providers` features:
  - `local-inference`
  - `cuda`
  - `vulkan`

This should be last because it has the most non-provider-runtime coupling.

---

# Registry and Initialization Plan

Current `providers/init.rs` owns a global `ProviderRegistry` and calls `register_declarative_providers`, which currently lives under `goose::config`.

## Proposed split

### In `goose-providers`

- `ProviderRegistry`
- `ProviderEntry`
- builtin provider registration function
- declarative provider schema and construction logic
- provider metadata

But registry constructors should accept `ProviderInit`/`ProviderRuntime`.

### In `goose`

- global registry singleton
- loading custom provider files from goose config dirs
- creating `ProviderRuntime`
- converting goose `ExtensionConfig` to provider DTOs
- server/CLI integration
- inventory refresh persistence

This keeps `goose-providers` independent and reusable while preserving goose-specific config storage and lifecycle.

---

# Declarative Providers

`DeclarativeProviderConfig` currently lives under `goose::config` but imports provider types and provider implementations. That creates a cycle once providers move.

## Plan

Move provider schema pieces to `goose-providers`:

- `DeclarativeProviderConfig`
- `ProviderEngine`
- `EnvVarConfig`
- expansion logic, but use injected config instead of `Config::global`

Keep goose-specific management APIs in `goose`:

- custom provider file location
- create/update/delete custom provider files
- UI/server routes
- active provider config

`goose` loads declarative configs and passes them to `goose-providers::ProviderRegistry`.

---

# Inventory Plan

Current inventory mixes provider identity/config detection with session storage refresh.

## Move to `goose-providers`

- `InventoryIdentityInput`
- inventory identity descriptors
- default identity helper, parameterized by config store
- `inventory_configured` helper, parameterized by config store
- provider metadata inventory fields

## Keep or adapt in `goose`

- refresh service that reads/writes `SessionStorage`
- scheduled refresh orchestration
- server routes around inventory
- catalog setup categories if tightly coupled to goose UI

---

# Cargo / Feature Plan

Add new crate:

```toml
[package]
name = "goose-providers"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
...

[features]
default = []
aws-providers = [...]
local-inference = [...]
cuda = ["local-inference", ...]
vulkan = ["local-inference", ...]
rustls-tls = [...]
native-tls = [...]
```

Then in `crates/goose/Cargo.toml`:

```toml
goose-providers = { path = "../goose-providers", default-features = false }
```

Feature forwarding:

```toml
aws-providers = ["goose-providers/aws-providers", ...]
local-inference = ["goose-providers/local-inference", ...]
rustls-tls = ["goose-providers/rustls-tls", ...]
native-tls = ["goose-providers/native-tls", ...]
```

Eventually provider-specific dependencies should move from `goose` to `goose-providers`.

---

# Compatibility Strategy

To reduce churn:

1. Move types to `goose-providers`.
2. Reexport them from `goose` at their old paths temporarily.

Example:

```rust
pub mod model {
    pub use goose_providers::model::*;
}
```

For `goose::conversation`, we can either:

- fully reexport `goose_providers::conversation`, or
- keep goose conversation orchestration while reusing provider message types.

This allows existing `goose-cli`, `goose-server`, and UI integration to migrate gradually.

---

# Important Design Decisions

## 1. Do not let `goose-providers` know about goose config

Even if we keep environment variable precedence behavior, that belongs in the injected `ProviderConfigStore` implementation in `goose`.

Providers should ask:

```rust
runtime.config.get_secret::<String>("OPENAI_API_KEY")
```

They should not know whether the source is env var, YAML, keyring, server-supplied config, or test fixture.

## 2. Make provider construction explicit

`from_env` should become `from_init` or `from_runtime`.

The old name bakes in an implementation detail and encourages hidden global reads.

## 3. Keep `Provider` API concrete, not over-abstracted

Move `Message`, `ModelConfig`, and `ProviderUsage` into `goose-providers` rather than adding conversion traits everywhere. These are central provider API types.

## 4. Split goose orchestration from provider capability

Session storage, active provider selection, UI setup flows, and agent extension resolution should stay in `goose`.

Provider request formatting, HTTP clients, OAuth flows, model metadata, and provider implementations should live in `goose-providers`.

---

# Major Risks

## ACP providers

ACP currently appears tightly coupled to goose internals. We should either:

- introduce `AcpSessionFactory`, or
- leave ACP providers in `goose` temporarily while the API crate lands.

Trying to move all ACP internals immediately will likely balloon the scope.

## Local inference

Local inference depends on paths, downloads, optional heavy deps, model registry, and prompt templates. It should be last.

## Message type movement

`Message` is used broadly across goose. Moving it is the right dependency direction, but it will touch many files. Reexports can make this manageable.

## Config semantics

Current config behavior includes env precedence, keyring fallback, grouped secret reads, cache invalidation, and specific error handling. The injected config trait must preserve these behaviors.

---

# Proposed End State

Dependency direction:

```text
goose-cli ─┐
goose-server ─┬─> goose ──> goose-providers
goose-mcp ────┘          └─> goose-sdk / other crates

goose-providers ──> external crates only
                  ├─ rmcp
                  ├─ reqwest
                  ├─ serde
                  ├─ tokio
                  ├─ futures
                  └─ provider SDK deps behind features
```

`goose-providers` contains:

- provider trait/API
- provider implementations
- request/response formatters
- provider errors/retry
- provider metadata/canonical models
- provider-owned message/model types
- provider runtime abstraction traits

`goose` contains:

- config storage
- session management
- agent orchestration
- extension resolution
- UI/server/CLI integration
- implementations of injected runtime services

---

# Concrete First PR Recommendation

A good first PR should avoid moving every provider immediately. I would do:

1. Add `crates/goose-providers`.
2. Move:
   - `providers/errors.rs`
   - `providers/retry.rs`
   - `providers/base.rs` core types, with goose dependencies removed/stubbed behind traits
   - `model.rs`, made pure
   - conversation message types needed by `Provider`
3. Add `ProviderConfigStore`, `ProviderPaths`, and `ProviderRuntime`.
4. Add goose adapter implementations.
5. Reexport moved types from `goose`.
6. Keep existing provider implementations in `goose` temporarily but make them compile against `goose_providers` types.

Then subsequent PRs can move providers family by family. This keeps the architectural boundary real without requiring a risky all-at-once migration.

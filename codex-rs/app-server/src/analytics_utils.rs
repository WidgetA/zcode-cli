use std::sync::Arc;

use codex_analytics::AnalyticsEventsClient;
use codex_core::config::Config;
use codex_login::AuthManager;

pub(crate) fn analytics_events_client_from_config(
    auth_manager: Arc<AuthManager>,
    config: &Config,
) -> AnalyticsEventsClient {
    AnalyticsEventsClient::new(
        auth_manager,
        config.chatgpt_base_url.trim_end_matches('/').to_string(),
        // zcode-cli fork: analytics events post to OpenAI's backend, so they
        // are opt-in (`analytics.enabled = true`) rather than on by default.
        Some(config.analytics_enabled.unwrap_or(false)),
    )
}

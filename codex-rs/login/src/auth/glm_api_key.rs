use std::path::Path;

use codex_config::types::AuthCredentialsStoreMode;

use super::manager::load_auth_dot_json;
use super::manager::save_auth;
use super::storage::AuthDotJson;
use super::storage::AuthKeyringBackendKind;

/// Persists a GLM (Zhipu) API key in the auth store.
///
/// Unlike the other `login_with_*` helpers this merges into any existing
/// `auth.json` payload instead of replacing it, so a stored ChatGPT login
/// survives `zcode login` with the GLM provider active. The stored key is
/// only used as a fallback when neither `ZHIPU_API_KEY` nor `ZCODE_API_KEY`
/// is set in the environment.
pub fn login_with_glm_api_key(
    codex_home: &Path,
    api_key: &str,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<()> {
    let mut auth_dot_json =
        load_auth_dot_json(codex_home, auth_credentials_store_mode, keyring_backend_kind)?
            .unwrap_or(AuthDotJson {
                auth_mode: None,
                openai_api_key: None,
                tokens: None,
                last_refresh: None,
                agent_identity: None,
                personal_access_token: None,
                bedrock_api_key: None,
                glm_api_key: None,
            });
    auth_dot_json.glm_api_key = Some(api_key.to_string());
    save_auth(
        codex_home,
        &auth_dot_json,
        auth_credentials_store_mode,
        keyring_backend_kind,
    )
}

/// Loads the GLM (Zhipu) API key persisted by [`login_with_glm_api_key`], if any.
pub fn load_stored_glm_api_key(
    codex_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<Option<String>> {
    Ok(
        load_auth_dot_json(codex_home, auth_credentials_store_mode, keyring_backend_kind)?
            .and_then(|auth| auth.glm_api_key)
            .filter(|api_key| !api_key.trim().is_empty()),
    )
}

#[cfg(test)]
#[path = "glm_api_key_tests.rs"]
mod tests;

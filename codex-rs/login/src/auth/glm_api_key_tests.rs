use pretty_assertions::assert_eq;
use tempfile::tempdir;

use super::load_stored_glm_api_key;
use super::login_with_glm_api_key;
use crate::AuthCredentialsStoreMode;
use crate::AuthKeyringBackendKind;
use crate::load_auth_dot_json;
use crate::logout;

#[test]
fn login_with_glm_api_key_saves_and_loads_key() {
    let codex_home = tempdir().expect("create temporary Codex home");

    login_with_glm_api_key(
        codex_home.path(),
        "glm-test-key",
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )
    .expect("save GLM API key");

    let stored = load_stored_glm_api_key(
        codex_home.path(),
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )
    .expect("load GLM API key");
    assert_eq!(stored, Some("glm-test-key".to_string()));

    // The key is persisted under the GLM_API_KEY field in auth.json.
    let raw = std::fs::read_to_string(codex_home.path().join("auth.json"))
        .expect("read auth.json");
    let json: serde_json::Value = serde_json::from_str(&raw).expect("parse auth.json");
    assert_eq!(json["GLM_API_KEY"], serde_json::json!("glm-test-key"));
}

#[test]
fn login_with_glm_api_key_preserves_existing_credentials() {
    let codex_home = tempdir().expect("create temporary Codex home");
    crate::login_with_api_key(
        codex_home.path(),
        "sk-openai",
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )
    .expect("save OpenAI API key");

    login_with_glm_api_key(
        codex_home.path(),
        "glm-test-key",
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )
    .expect("save GLM API key");

    let auth = load_auth_dot_json(
        codex_home.path(),
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )
    .expect("load auth")
    .expect("auth should exist");
    assert_eq!(auth.openai_api_key, Some("sk-openai".to_string()));
    assert_eq!(auth.glm_api_key, Some("glm-test-key".to_string()));
}

#[test]
fn load_stored_glm_api_key_returns_none_without_login() {
    let codex_home = tempdir().expect("create temporary Codex home");

    let stored = load_stored_glm_api_key(
        codex_home.path(),
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )
    .expect("load GLM API key");
    assert_eq!(stored, None);
}

#[test]
fn load_stored_glm_api_key_ignores_empty_key() {
    let codex_home = tempdir().expect("create temporary Codex home");
    login_with_glm_api_key(
        codex_home.path(),
        "   ",
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )
    .expect("save blank GLM API key");

    let stored = load_stored_glm_api_key(
        codex_home.path(),
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )
    .expect("load GLM API key");
    assert_eq!(stored, None);
}

#[test]
fn logout_clears_stored_glm_api_key() {
    let codex_home = tempdir().expect("create temporary Codex home");
    login_with_glm_api_key(
        codex_home.path(),
        "glm-test-key",
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )
    .expect("save GLM API key");

    let removed = logout(
        codex_home.path(),
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )
    .expect("logout");
    assert!(removed);

    let stored = load_stored_glm_api_key(
        codex_home.path(),
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )
    .expect("load GLM API key");
    assert_eq!(stored, None);
}

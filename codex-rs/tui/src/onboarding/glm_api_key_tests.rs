use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;
use ratatui::Terminal;
use tempfile::TempDir;

use super::GlmApiKeyOutcome;
use super::GlmApiKeyWidget;
use super::glm_api_key_setup_needed;
use crate::legacy_core::config::Config;
use crate::legacy_core::config::ConfigBuilder;
use crate::onboarding::onboarding_screen::KeyboardHandler;
use crate::onboarding::onboarding_screen::StepState;
use crate::onboarding::onboarding_screen::StepStateProvider;
use crate::test_backend::VT100Backend;
use crate::tui::FrameRequester;

/// Restores the GLM credential env vars on drop so these tests are
/// independent of the developer/CI environment.
struct GlmEnvGuard {
    zhipu_api_key: Option<std::ffi::OsString>,
    zcode_api_key: Option<std::ffi::OsString>,
}

impl GlmEnvGuard {
    fn remove_vars() -> Self {
        let guard = Self {
            zhipu_api_key: std::env::var_os("ZHIPU_API_KEY"),
            zcode_api_key: std::env::var_os("ZCODE_API_KEY"),
        };
        unsafe {
            std::env::remove_var("ZHIPU_API_KEY");
            std::env::remove_var("ZCODE_API_KEY");
        }
        guard
    }
}

impl Drop for GlmEnvGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.zhipu_api_key {
                Some(value) => std::env::set_var("ZHIPU_API_KEY", value),
                None => std::env::remove_var("ZHIPU_API_KEY"),
            }
            match &self.zcode_api_key {
                Some(value) => std::env::set_var("ZCODE_API_KEY", value),
                None => std::env::remove_var("ZCODE_API_KEY"),
            }
        }
    }
}

async fn build_config(temp_dir: &TempDir) -> std::io::Result<Config> {
    ConfigBuilder::default()
        .codex_home(temp_dir.path().to_path_buf())
        .build()
        .await
}

fn stored_key(codex_home: &std::path::Path) -> Option<String> {
    codex_login::load_stored_glm_api_key(
        codex_home,
        codex_login::AuthCredentialsStoreMode::File,
        codex_login::AuthKeyringBackendKind::default(),
    )
    .expect("load stored GLM API key")
}

#[tokio::test]
async fn setup_needed_without_any_key_and_not_after_storing_one() {
    let _env_guard = GlmEnvGuard::remove_vars();
    let temp_dir = TempDir::new().expect("temp dir");
    let config = build_config(&temp_dir).await.expect("config");
    assert!(config.model_provider.is_glm());

    assert!(glm_api_key_setup_needed(&config));

    codex_login::login_with_glm_api_key(
        temp_dir.path(),
        "glm-test-key",
        codex_login::AuthCredentialsStoreMode::File,
        codex_login::AuthKeyringBackendKind::default(),
    )
    .expect("save GLM API key");
    assert!(!glm_api_key_setup_needed(&config));
}

#[tokio::test]
async fn setup_not_needed_with_env_key() {
    let _env_guard = GlmEnvGuard::remove_vars();
    let temp_dir = TempDir::new().expect("temp dir");
    let config = build_config(&temp_dir).await.expect("config");

    unsafe { std::env::set_var("ZHIPU_API_KEY", "glm-env-key") };
    assert!(!glm_api_key_setup_needed(&config));
}

#[tokio::test]
async fn enter_saves_key_and_completes_step() {
    let _env_guard = GlmEnvGuard::remove_vars();
    let temp_dir = TempDir::new().expect("temp dir");
    let config = build_config(&temp_dir).await.expect("config");
    let mut widget = GlmApiKeyWidget::new(&config, FrameRequester::test_dummy());

    for c in "glm-typed-key".chars() {
        widget.handle_key_event(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }
    assert_eq!(widget.get_step_state(), StepState::InProgress);

    widget.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_eq!(widget.outcome, Some(GlmApiKeyOutcome::Saved));
    assert_eq!(widget.get_step_state(), StepState::Complete);
    assert_eq!(
        stored_key(temp_dir.path()),
        Some("glm-typed-key".to_string())
    );
}

#[tokio::test]
async fn enter_with_empty_input_shows_error() {
    let _env_guard = GlmEnvGuard::remove_vars();
    let temp_dir = TempDir::new().expect("temp dir");
    let config = build_config(&temp_dir).await.expect("config");
    let mut widget = GlmApiKeyWidget::new(&config, FrameRequester::test_dummy());

    widget.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_eq!(widget.outcome, None);
    assert_eq!(widget.error, Some("API key cannot be empty".to_string()));
    assert_eq!(widget.get_step_state(), StepState::InProgress);
    assert_eq!(stored_key(temp_dir.path()), None);
}

#[tokio::test]
async fn esc_skips_without_saving() {
    let _env_guard = GlmEnvGuard::remove_vars();
    let temp_dir = TempDir::new().expect("temp dir");
    let config = build_config(&temp_dir).await.expect("config");
    let mut widget = GlmApiKeyWidget::new(&config, FrameRequester::test_dummy());

    widget.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert_eq!(widget.outcome, Some(GlmApiKeyOutcome::Skipped));
    assert_eq!(widget.get_step_state(), StepState::Complete);
    assert_eq!(stored_key(temp_dir.path()), None);
}

#[tokio::test]
async fn paste_populates_input() {
    let _env_guard = GlmEnvGuard::remove_vars();
    let temp_dir = TempDir::new().expect("temp dir");
    let config = build_config(&temp_dir).await.expect("config");
    let mut widget = GlmApiKeyWidget::new(&config, FrameRequester::test_dummy());

    widget.handle_paste("  glm-pasted-key\n".to_string());

    assert_eq!(widget.value, "glm-pasted-key");
}

#[tokio::test]
async fn renders_snapshot_for_api_key_entry() {
    let _env_guard = GlmEnvGuard::remove_vars();
    let temp_dir = TempDir::new().expect("temp dir");
    let config = build_config(&temp_dir).await.expect("config");
    let mut widget = GlmApiKeyWidget::new(&config, FrameRequester::test_dummy());
    widget.handle_paste("glm-test-key".to_string());

    let mut terminal =
        Terminal::new(VT100Backend::new(/*width*/ 70, /*height*/ 14)).expect("terminal");
    terminal
        .draw(|f| ratatui::widgets::WidgetRef::render_ref(&&widget, f.area(), f.buffer_mut()))
        .expect("draw");

    insta::assert_snapshot!(terminal.backend());
}

#[tokio::test]
async fn renders_snapshot_for_saved_key() {
    let _env_guard = GlmEnvGuard::remove_vars();
    let temp_dir = TempDir::new().expect("temp dir");
    let config = build_config(&temp_dir).await.expect("config");
    let mut widget = GlmApiKeyWidget::new(&config, FrameRequester::test_dummy());
    widget.handle_paste("glm-test-key".to_string());
    widget.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(widget.outcome, Some(GlmApiKeyOutcome::Saved));

    let mut terminal =
        Terminal::new(VT100Backend::new(/*width*/ 70, /*height*/ 6)).expect("terminal");
    terminal
        .draw(|f| ratatui::widgets::WidgetRef::render_ref(&&widget, f.area(), f.buffer_mut()))
        .expect("draw");

    insta::assert_snapshot!(terminal.backend());
}

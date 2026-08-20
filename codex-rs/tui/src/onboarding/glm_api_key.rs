//! First-run GLM (Zhipu) API key onboarding step.
//!
//! When the GLM provider is active but no API key is available from the
//! environment (`ZHIPU_API_KEY`/`ZCODE_API_KEY`) or the auth store, this step
//! asks the user to paste a key before they reach the composer. Enter persists
//! the key via the auth store; Esc skips and lets the user continue without a
//! key (requests then fail with the missing-key error until `zcode login` is
//! run).

use std::path::PathBuf;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::WidgetRef;
use ratatui::widgets::Wrap;

use codex_login::AuthCredentialsStoreMode;
use codex_login::AuthKeyringBackendKind;

use crate::key_hint::KeyBindingListExt;
use crate::legacy_core::config::Config;
use crate::onboarding::keys;
use crate::onboarding::onboarding_screen::KeyboardHandler;
use crate::onboarding::onboarding_screen::StepState;
use crate::onboarding::onboarding_screen::StepStateProvider;
use crate::tui::FrameRequester;

/// Returns true when the GLM provider is active but no API key is available
/// from the environment or the auth store, so first-run onboarding should ask
/// for one.
pub(crate) fn glm_api_key_setup_needed(config: &Config) -> bool {
    config.model_provider.is_glm()
        && !matches!(config.model_provider.api_key(), Ok(Some(_)))
        && !has_stored_glm_api_key(config)
}

fn has_stored_glm_api_key(config: &Config) -> bool {
    codex_login::load_stored_glm_api_key(
        &config.codex_home,
        config.cli_auth_credentials_store_mode,
        config.auth_keyring_backend_kind(),
    )
    .ok()
    .flatten()
    .is_some()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GlmApiKeyOutcome {
    Saved,
    Skipped,
}

pub(crate) struct GlmApiKeyWidget {
    request_frame: FrameRequester,
    codex_home: PathBuf,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
    value: String,
    error: Option<String>,
    outcome: Option<GlmApiKeyOutcome>,
}

impl GlmApiKeyWidget {
    pub(crate) fn new(config: &Config, request_frame: FrameRequester) -> Self {
        Self {
            request_frame,
            codex_home: config.codex_home.to_path_buf(),
            auth_credentials_store_mode: config.cli_auth_credentials_store_mode,
            keyring_backend_kind: config.auth_keyring_backend_kind(),
            value: String::new(),
            error: None,
            outcome: None,
        }
    }

    /// True while the user is still editing the API-key field.
    pub(crate) fn is_entry_active(&self) -> bool {
        self.outcome.is_none()
    }

    /// True when the API-key input field currently contains user text.
    pub(crate) fn entry_has_text(&self) -> bool {
        !self.value.is_empty()
    }

    fn save_api_key(&mut self) {
        let api_key = self.value.trim().to_string();
        if api_key.is_empty() {
            self.error = Some("API key cannot be empty".to_string());
            return;
        }
        match codex_login::login_with_glm_api_key(
            &self.codex_home,
            &api_key,
            self.auth_credentials_store_mode,
            self.keyring_backend_kind,
        ) {
            Ok(()) => {
                self.error = None;
                self.outcome = Some(GlmApiKeyOutcome::Saved);
            }
            Err(err) => {
                self.error = Some(format!("Failed to save API key: {err}"));
            }
        }
    }
}

impl KeyboardHandler for GlmApiKeyWidget {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.outcome.is_some() || key_event.kind != KeyEventKind::Press {
            return;
        }
        if keys::CANCEL.is_pressed(key_event) {
            self.outcome = Some(GlmApiKeyOutcome::Skipped);
        } else if keys::CONFIRM.is_pressed(key_event) {
            self.save_api_key();
        } else {
            match key_event.code {
                KeyCode::Backspace => {
                    self.value.pop();
                    self.error = None;
                }
                KeyCode::Char(c)
                    if !key_event.modifiers.intersects(
                        KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                    ) =>
                {
                    self.value.push(c);
                    self.error = None;
                }
                _ => {}
            }
        }
        self.request_frame.schedule_frame();
    }

    fn handle_paste(&mut self, pasted: String) {
        if self.outcome.is_some() {
            return;
        }
        let trimmed = pasted.trim();
        if trimmed.is_empty() {
            return;
        }
        self.value.push_str(trimmed);
        self.error = None;
        self.request_frame.schedule_frame();
    }
}

impl StepStateProvider for GlmApiKeyWidget {
    fn get_step_state(&self) -> StepState {
        if self.outcome.is_some() {
            StepState::Complete
        } else {
            StepState::InProgress
        }
    }
}

impl WidgetRef for &GlmApiKeyWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        match self.outcome {
            Some(GlmApiKeyOutcome::Saved) => {
                Paragraph::new(vec![
                    "✓ GLM API key saved".green().into(),
                    "".into(),
                    "  Update it any time with `zcode login`.".into(),
                ])
                .wrap(Wrap { trim: false })
                .render(area, buf);
            }
            Some(GlmApiKeyOutcome::Skipped) => {}
            None => self.render_api_key_entry(area, buf),
        }
    }
}

impl GlmApiKeyWidget {
    fn render_api_key_entry(&self, area: Rect, buf: &mut Buffer) {
        let [intro_area, input_area, footer_area] = Layout::vertical([
            Constraint::Min(4),
            Constraint::Length(3),
            Constraint::Min(2),
        ])
        .areas(area);

        let intro_lines: Vec<Line> = vec![
            Line::from(vec!["> ".into(), "GLM API Key".bold()]),
            "".into(),
            "  zcode uses the GLM (Zhipu) provider, which needs an API key.".into(),
            Line::from(vec![
                "  Get a key at ".into(),
                "https://open.bigmodel.cn".cyan().underlined(),
                ".".into(),
            ]),
            "  It will be stored locally in auth.json.".dim().into(),
            "".into(),
        ];
        Paragraph::new(intro_lines)
            .wrap(Wrap { trim: false })
            .render(intro_area, buf);

        let content_line: Line = if self.value.is_empty() {
            vec!["Paste or type your API key".dim()].into()
        } else {
            Line::from(self.value.clone())
        };
        Paragraph::new(content_line)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title("GLM API key")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .render(input_area, buf);

        let mut footer_lines: Vec<Line> = vec![
            Line::from(vec![
                "  Press ".dim(),
                keys::CONFIRM[0].into(),
                " to save".dim(),
            ]),
            Line::from(vec![
                "  Press ".dim(),
                keys::CANCEL[0].into(),
                " to skip".dim(),
            ]),
        ];
        if let Some(error) = &self.error {
            footer_lines.push("".into());
            footer_lines.push(error.clone().red().into());
        }
        Paragraph::new(footer_lines)
            .wrap(Wrap { trim: false })
            .render(footer_area, buf);
    }
}

#[cfg(test)]
#[path = "glm_api_key_tests.rs"]
mod tests;

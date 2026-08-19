use codex_models_manager::model_info::BASE_INSTRUCTIONS;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::openai_models::ConfigShellToolType;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelMessages;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ModelsResponse;
use codex_protocol::openai_models::TruncationPolicyConfig;
use codex_protocol::openai_models::WebSearchToolType;
use codex_protocol::openai_models::default_input_modalities;

pub const GLM_DEFAULT_MODEL: &str = "glm-5.3";

const GLM_5_3_CONTEXT_WINDOW: i64 = 1_000_000;
const GLM_DEFAULT_CONTEXT_WINDOW: i64 = 200_000;

/// Static model catalog for the GLM (Zhipu) provider.
///
/// The GLM Coding Plan endpoint does not expose a Codex-compatible `/models`
/// catalog, so the provider ships an authoritative in-process catalog instead
/// of fetching one remotely.
pub(crate) fn static_model_catalog() -> ModelsResponse {
    ModelsResponse {
        models: vec![
            glm_model(
                GLM_DEFAULT_MODEL,
                "GLM-5.3",
                /*priority*/ 0,
                GLM_5_3_CONTEXT_WINDOW,
            ),
            glm_model(
                "glm-5.2",
                "GLM-5.2",
                /*priority*/ 1,
                GLM_DEFAULT_CONTEXT_WINDOW,
            ),
            glm_model(
                "glm-4.6",
                "GLM-4.6",
                /*priority*/ 2,
                GLM_DEFAULT_CONTEXT_WINDOW,
            ),
        ],
    }
}

fn glm_model(slug: &str, display_name: &str, priority: i32, context_window: i64) -> ModelInfo {
    ModelInfo {
        slug: slug.to_string(),
        display_name: display_name.to_string(),
        description: None,
        default_reasoning_level: None,
        supported_reasoning_levels: Vec::new(),
        shell_type: ConfigShellToolType::Default,
        visibility: ModelVisibility::List,
        supported_in_api: true,
        priority,
        additional_speed_tiers: Vec::new(),
        service_tiers: Vec::new(),
        default_service_tier: None,
        availability_nux: None,
        upgrade: None,
        model_messages: Some(ModelMessages {
            instructions_template: Some(BASE_INSTRUCTIONS.to_string()),
            instructions_variables: None,
            approvals: None,
            collaboration_modes: None,
            auto_review: None,
            permissions: None,
            multi_agent: None,
            token_budget: None,
            guardian_v2: None,
        }),
        include_skills_usage_instructions: false,
        include_plugin_usage_instructions: false,
        include_apps_usage_instructions: false,
        supports_reasoning_summary_parameter: true,
        default_reasoning_summary: ReasoningSummary::Auto,
        support_verbosity: false,
        default_verbosity: None,
        apply_patch_tool_type: None,
        web_search_tool_type: WebSearchToolType::Text,
        truncation_policy: TruncationPolicyConfig::bytes(/*limit*/ 10_000),
        supports_image_detail_original: false,
        context_window: Some(context_window),
        max_context_window: Some(context_window),
        auto_compact_token_limit: None,
        comp_hash: None,
        effective_context_window_percent: 95,
        experimental_supported_tools: Vec::new(),
        input_modalities: default_input_modalities(),
        used_fallback_model_metadata: false,
        supports_search_tool: false,
        use_responses_lite: false,
        node_repl_auto_review_required: false,
        node_repl_disabled: false,
        auto_review_model_override: None,
        model_specialty: None,
        tool_mode: None,
        multi_agent_version: None,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn catalog_marks_glm_5_3_as_lowest_priority_default_candidate() {
        let catalog = static_model_catalog();

        assert_eq!(
            catalog
                .models
                .iter()
                .map(|model| (model.slug.as_str(), model.priority))
                .collect::<Vec<_>>(),
            vec![(GLM_DEFAULT_MODEL, 0), ("glm-5.2", 1), ("glm-4.6", 2)]
        );
        for model in &catalog.models {
            assert_eq!(model.visibility, ModelVisibility::List);
            assert!(!model.used_fallback_model_metadata);
        }
    }

    #[test]
    fn catalog_uses_documented_context_windows() {
        let catalog = static_model_catalog();

        assert_eq!(
            catalog
                .models
                .iter()
                .map(|model| (model.slug.as_str(), model.context_window))
                .collect::<Vec<_>>(),
            vec![
                (GLM_DEFAULT_MODEL, Some(GLM_5_3_CONTEXT_WINDOW)),
                ("glm-5.2", Some(GLM_DEFAULT_CONTEXT_WINDOW)),
                ("glm-4.6", Some(GLM_DEFAULT_CONTEXT_WINDOW)),
            ]
        );
    }
}

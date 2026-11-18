use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vocabulary {
    pub version: String,
    pub description: String,
    pub intents: Vec<IntentDefinition>,
    pub parameters: ParameterDefinitions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentDefinition {
    pub intent_id: String,
    pub target_tool: String,
    pub is_write: bool,
    pub description: String,
    pub action_verbs: Vec<String>,
    pub entity_nouns: Vec<String>,
    pub phrases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinitions {
    pub employee_id: PatternParameter,
    pub departments: LookupParameter,
    pub designations: LookupParameter,
    pub statuses: EnumParameter,
    pub email: SinglePatternParameter,
    pub salary: PatternParameter,
    pub phone: SinglePatternParameter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternParameter {
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: Option<String>,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinglePatternParameter {
    #[serde(rename = "type")]
    pub param_type: String,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupParameter {
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    pub canonical_mapping: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub department_id_mapping: Option<HashMap<String, String>>,
    #[serde(default)]
    pub designation_id_mapping: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumParameter {
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    pub canonical_mapping: HashMap<String, Vec<String>>,
}

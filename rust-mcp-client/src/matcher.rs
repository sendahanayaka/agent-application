use crate::models::Vocabulary;
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;

/// Helper for matching terms with boundary conditions and word variations
pub struct VocabularyMatcher {
    vocab: Vocabulary,
    compiled_patterns: HashMap<String, Vec<Regex>>,
}

impl VocabularyMatcher {
    pub fn new(vocab: Vocabulary) -> Result<Self, regex::Error> {
        let mut compiled = HashMap::new();

        // Compile regexes for each intent dynamically from verbs + nouns
        for intent in &vocab.intents {
            let mut list = Vec::new();
            for verb in &intent.action_verbs {
                for noun in &intent.entity_nouns {
                    // Pattern: \b<verb>\b.*\b<noun>\b
                    let pattern_str = format!(
                        r"(?i)\b{}\b.*\b{}\b",
                        regex::escape(verb),
                        regex::escape(noun)
                    );
                    let re = RegexBuilder::new(&pattern_str)
                        .case_insensitive(true)
                        .build()?;
                    list.push(re);
                }
            }
            compiled.insert(intent.intent_id.clone(), list);
        }

        Ok(Self {
            vocab,
            compiled_patterns: compiled,
        })
    }

    pub fn vocabulary(&self) -> &Vocabulary {
        &self.vocab
    }

    /// Checks if a prompt matches an intent using its action verbs and entity nouns
    pub fn matches_intent(&self, intent_id: &str, prompt: &str) -> bool {
        if let Some(regexes) = self.compiled_patterns.get(intent_id) {
            for re in regexes {
                if re.is_match(prompt) {
                    return true;
                }
            }
        }
        false
    }

    /// Resolves canonical department name from user text (longest/most-specific match first)
    pub fn resolve_department(&self, input: &str) -> Option<String> {
        let lower = input.to_lowercase();
        let mut candidates = Vec::new();
        for (canonical, aliases) in &self.vocab.parameters.departments.canonical_mapping {
            if lower == canonical.to_lowercase() {
                candidates.push((canonical.len(), canonical.clone()));
            }
            for alias in aliases {
                let pattern = format!(r"(?i)\b{}\b", regex::escape(alias));
                if let Ok(re) = Regex::new(&pattern) {
                    if re.is_match(&lower) {
                        candidates.push((alias.len(), canonical.clone()));
                    }
                }
            }
        }
        candidates.sort_by(|a, b| b.0.cmp(&a.0));
        candidates.into_iter().next().map(|(_, c)| c)
    }

    /// Resolves canonical designation title from user text (longest/most-specific match first)
    pub fn resolve_designation(&self, input: &str) -> Option<String> {
        let lower = input.to_lowercase();
        let mut candidates = Vec::new();
        for (canonical, aliases) in &self.vocab.parameters.designations.canonical_mapping {
            if lower == canonical.to_lowercase() {
                candidates.push((canonical.len(), canonical.clone()));
            }
            for alias in aliases {
                let pattern = format!(r"(?i)\b{}\b", regex::escape(alias));
                if let Ok(re) = Regex::new(&pattern) {
                    if re.is_match(&lower) {
                        candidates.push((alias.len(), canonical.clone()));
                    }
                }
            }
        }
        candidates.sort_by(|a, b| b.0.cmp(&a.0));
        candidates.into_iter().next().map(|(_, c)| c)
    }

    /// Resolves canonical employment status from user text (longest/most-specific match first)
    pub fn resolve_status(&self, input: &str) -> Option<String> {
        let lower = input.to_lowercase();
        let mut candidates = Vec::new();
        for (canonical, aliases) in &self.vocab.parameters.statuses.canonical_mapping {
            if lower == canonical.to_lowercase() {
                candidates.push((canonical.len(), canonical.clone()));
            }
            for alias in aliases {
                let pattern = format!(r"(?i)\b{}\b", regex::escape(alias));
                if let Ok(re) = Regex::new(&pattern) {
                    if re.is_match(&lower) {
                        candidates.push((alias.len(), canonical.clone()));
                    }
                }
            }
        }
        candidates.sort_by(|a, b| b.0.cmp(&a.0));
        candidates.into_iter().next().map(|(_, c)| c)
    }

    /// Extracts employee ID from prompt using configured patterns
    pub fn extract_employee_id(&self, prompt: &str) -> Option<i32> {
        for pattern_str in &self.vocab.parameters.employee_id.patterns {
            if let Ok(re) = RegexBuilder::new(pattern_str).case_insensitive(true).build() {
                if let Some(caps) = re.captures(prompt) {
                    if let Some(matched) = caps.get(1) {
                        if let Ok(id) = matched.as_str().parse::<i32>() {
                            return Some(id);
                        }
                    }
                }
            }
        }
        None
    }

    /// Extracts email address from prompt
    pub fn extract_email(&self, prompt: &str) -> Option<String> {
        if let Ok(re) = Regex::new(&self.vocab.parameters.email.pattern) {
            if let Some(caps) = re.captures(prompt) {
                if let Some(matched) = caps.get(1) {
                    return Some(matched.as_str().to_string());
                }
            }
        }
        None
    }
}

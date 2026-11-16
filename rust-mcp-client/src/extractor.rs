use crate::matcher::VocabularyMatcher;
use crate::models::Vocabulary;
use regex::{Regex, RegexBuilder};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct ParameterExtractor<'a> {
    matcher: &'a VocabularyMatcher,
    vocab: &'a Vocabulary,
}

impl<'a> ParameterExtractor<'a> {
    pub fn new(matcher: &'a VocabularyMatcher) -> Self {
        let vocab = matcher.vocabulary();
        Self { matcher, vocab }
    }

    /// Extract all identifiable parameters from a prompt segment
    pub fn extract_all(&self, text: &str) -> HashMap<String, Value> {
        let mut params = HashMap::new();

        if let Some(id) = self.matcher.extract_employee_id(text) {
            params.insert("employee_id".to_string(), json!(id));
        }

        if let Some(dept) = self.matcher.resolve_department(text) {
            params.insert("department".to_string(), json!(dept));
            if let Some(dept_id) = self.lookup_department_id(&dept) {
                params.insert("department_id".to_string(), json!(dept_id));
            }
        }

        if let Some(desig) = self.matcher.resolve_designation(text) {
            params.insert("designation".to_string(), json!(desig));
            if let Some(desig_id) = self.lookup_designation_id(&desig) {
                params.insert("designation_id".to_string(), json!(desig_id));
            }
        }

        if let Some(status) = self.matcher.resolve_status(text) {
            params.insert("status".to_string(), json!(status));
        }

        if let Some(email) = self.matcher.extract_email(text) {
            params.insert("email".to_string(), json!(email));
        }

        if let Some(salary) = self.extract_salary(text) {
            params.insert("salary".to_string(), json!(salary));
        }

        if let Some(phone) = self.extract_phone(text) {
            params.insert("phone".to_string(), json!(phone));
        }

        if let Some((first, last)) = self.extract_name(text) {
            params.insert("first_name".to_string(), json!(first));
            params.insert("last_name".to_string(), json!(last));
            params.insert("name".to_string(), json!(format!("{} {}", first, last)));
        } else if let Some(single_name) = self.extract_search_name(text) {
            params.insert("name".to_string(), json!(single_name));
        }

        params
    }

    pub fn lookup_department_id(&self, canonical_dept: &str) -> Option<i32> {
        if let Some(mapping) = &self.vocab.parameters.departments.department_id_mapping {
            for (id_str, name) in mapping {
                if name.eq_ignore_ascii_case(canonical_dept) {
                    if let Ok(id) = id_str.parse::<i32>() {
                        return Some(id);
                    }
                }
            }
        }
        None
    }

    pub fn lookup_designation_id(&self, canonical_desig: &str) -> Option<i32> {
        if let Some(mapping) = &self.vocab.parameters.designations.designation_id_mapping {
            for (id_str, name) in mapping {
                if name.eq_ignore_ascii_case(canonical_desig) {
                    if let Ok(id) = id_str.parse::<i32>() {
                        return Some(id);
                    }
                }
            }
        }
        None
    }

    pub fn extract_salary(&self, text: &str) -> Option<f64> {
        for pattern in &self.vocab.parameters.salary.patterns {
            if let Ok(re) = RegexBuilder::new(pattern).case_insensitive(true).build() {
                if let Some(caps) = re.captures(text) {
                    if let Some(m) = caps.get(1) {
                        let cleaned = m.as_str().replace(',', "");
                        if let Ok(val) = cleaned.parse::<f64>() {
                            return Some(val);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn extract_phone(&self, text: &str) -> Option<String> {
        if let Ok(re) = RegexBuilder::new(&self.vocab.parameters.phone.pattern)
            .case_insensitive(true)
            .build()
        {
            if let Some(caps) = re.captures(text) {
                if let Some(m) = caps.get(1) {
                    let cleaned = m.as_str().trim().to_string();
                    if cleaned.len() >= 7 {
                        return Some(cleaned);
                    }
                }
            }
        }
        None
    }

    pub fn extract_name(&self, text: &str) -> Option<(String, String)> {
        // e.g. "named John Doe", "name is John Doe", "employee John Doe"
        let patterns = [
            r"(?i)\bnamed\s+([A-Z][a-z]+)\s+([A-Z][a-z]+)\b",
            r"(?i)\bname\s+(?:is|:)?\s*([A-Z][a-z]+)\s+([A-Z][a-z]+)\b",
            r"(?i)\b(?:add|create|insert|hire)\s+(?:employee|new employee|person)?\s+([A-Z][a-z]+)\s+([A-Z][a-z]+)\b",
        ];

        for pat in patterns {
            if let Ok(re) = Regex::new(pat) {
                if let Some(caps) = re.captures(text) {
                    let first = caps.get(1).map(|m| m.as_str().to_string());
                    let last = caps.get(2).map(|m| m.as_str().to_string());
                    if let (Some(f), Some(l)) = (first, last) {
                        return Some((f, l));
                    }
                }
            }
        }
        None
    }

    pub fn extract_search_name(&self, text: &str) -> Option<String> {
        let patterns = [
            r"(?i)\bnamed\s+([A-Z][a-z]+)\b",
            r"(?i)\b(?:search for|find|lookup)\s+([A-Z][a-z]+)\b",
        ];

        for pat in patterns {
            if let Ok(re) = Regex::new(pat) {
                if let Some(caps) = re.captures(text) {
                    if let Some(m) = caps.get(1) {
                        let candidate = m.as_str();
                        // avoid collision with keywords
                        let lower = candidate.to_lowercase();
                        if lower != "employee" && lower != "staff" && lower != "department" && lower != "all" {
                            return Some(candidate.to_string());
                        }
                    }
                }
            }
        }
        None
    }
}

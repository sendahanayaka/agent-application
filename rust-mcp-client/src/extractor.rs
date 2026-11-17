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
        } else if let Some(unmatched_dept) = self.extract_unmatched_department(text) {
            params.insert("department".to_string(), json!(unmatched_dept));
        }

        if let Some(desig) = self.matcher.resolve_designation(text) {
            params.insert("designation".to_string(), json!(desig));
            if let Some(desig_id) = self.lookup_designation_id(&desig) {
                params.insert("designation_id".to_string(), json!(desig_id));
            }
        } else if let Some(unmatched_desig) = self.extract_unmatched_designation(text) {
            params.insert("designation".to_string(), json!(unmatched_desig));
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
        } else if !params.contains_key("department") && !params.contains_key("designation") && !params.contains_key("status") {
            // Only search for a standalone name if neither department, designation, nor status was matched
            if let Some(single_name) = self.extract_search_name(text) {
                params.insert("name".to_string(), json!(single_name));
            }
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
        // If an explicit negative salary is specified (e.g. "salary -500" or "pay -95000"), reject it
        let neg_re = Regex::new(r"(?i)\b(?:salary|compensation|pay|rate)\s*(?:is|to|:|of)?\s*-\s*[0-9]+").unwrap();
        if neg_re.is_match(text) {
            return None;
        }

        for pattern in &self.vocab.parameters.salary.patterns {
            if let Ok(re) = RegexBuilder::new(pattern).case_insensitive(true).build() {
                if let Some(caps) = re.captures(text) {
                    if let Some(m) = caps.get(1) {
                        let cleaned = m.as_str().replace(',', "");
                        if let Ok(val) = cleaned.parse::<f64>() {
                            if val > 0.0 {
                                return Some(val);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn extract_phone(&self, text: &str) -> Option<String> {
        let patterns = [
            &self.vocab.parameters.phone.pattern,
            r"(?i)\b(?:phone\s*(?:number)?|mobile|cell|tel)\s*(?:of\s+employee\s+[0-9]+\s+)?(?:is|to|:|as|=)?\s*([0-9+()\-.\s]{7,20})\b",
            r"(?i)\b(?:set|update|change|add)?\s*phone\s*(?:number)?\s*(?:to|as|=|:)?\s*([0-9+()\-.\s]{7,20})\b",
        ];

        for pat in &patterns {
            if let Ok(re) = RegexBuilder::new(pat)
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

    pub fn extract_unmatched_department(&self, text: &str) -> Option<String> {
        let patterns = [
            r"(?i)\b(?:in|of|for)\s+(?:a\s+|an\s+|the\s+)?([A-Za-z0-9_-]+)\s+department\b",
            r"(?i)\bdepartment\s+(?:named|of|called|:)?\s*([A-Za-z0-9_-]+)\b",
            r"(?i)\b(?:in|of|for)\s+(?:a\s+|an\s+|the\s+)?([A-Za-z0-9_-]+)\s+dept\b",
        ];
        for pat in patterns {
            if let Ok(re) = Regex::new(pat) {
                if let Some(caps) = re.captures(text) {
                    if let Some(m) = caps.get(1) {
                        let val = m.as_str().trim();
                        if !val.eq_ignore_ascii_case("the") && !val.eq_ignore_ascii_case("all") && !val.eq_ignore_ascii_case("a") && !val.eq_ignore_ascii_case("an") {
                            return Some(val.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    pub fn extract_unmatched_designation(&self, text: &str) -> Option<String> {
        let patterns = [
            r"(?i)\b(?:with\s+(?:a\s+|an\s+|the\s+)?designation|designation\s+(?:is|of|called|:)?)\s+([A-Za-z0-9_\s-]+?)(?:\s+in\b|\s+with\b|\s+at\b|$)",
            r"(?i)\b(?:with\s+(?:a\s+|an\s+|the\s+)?role|role\s+(?:is|of|called|:)?)\s+([A-Za-z0-9_\s-]+?)(?:\s+in\b|\s+with\b|\s+at\b|$)",
            r"(?i)\b(?:with\s+(?:a\s+|an\s+|the\s+)?title|title\s+(?:is|of|called|:)?)\s+([A-Za-z0-9_\s-]+?)(?:\s+in\b|\s+with\b|\s+at\b|$)",
            r"(?i)\b(?:with\s+(?:a\s+|an\s+|the\s+)?|having\s+(?:a\s+|an\s+|the\s+)?)([A-Za-z0-9_-]+)\s+designation\b",
            r"(?i)\b(?:with\s+(?:a\s+|an\s+|the\s+)?|having\s+(?:a\s+|an\s+|the\s+)?)([A-Za-z0-9_-]+)\s+role\b",
            r"(?i)\b(?:with\s+(?:a\s+|an\s+|the\s+)?|having\s+(?:a\s+|an\s+|the\s+)?)([A-Za-z0-9_-]+)\s+title\b",
        ];
        for pat in patterns {
            if let Ok(re) = Regex::new(pat) {
                if let Some(caps) = re.captures(text) {
                    if let Some(m) = caps.get(1) {
                        let val = m.as_str().trim();
                        if !val.is_empty() && !val.eq_ignore_ascii_case("the") && !val.eq_ignore_ascii_case("a") && !val.eq_ignore_ascii_case("an") {
                            return Some(val.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    pub fn extract_search_name(&self, text: &str) -> Option<String> {
        let patterns = [
            r"(?i)\bnamed\s+([A-Za-z]+)\b",
            r"(?i)\bname\s+(?:is|:)?\s*([A-Za-z]+)\b",
            r"(?i)\b(?:search for|find|lookup)\s+(?:employee\s+|staff\s+)?([A-Za-z]+)\b",
        ];

        let stop_words = [
            "employee", "employees", "staff", "department", "departments",
            "all", "who", "whom", "are", "with", "the", "in", "for", "at",
            "active", "inactive", "leave", "on", "terminated", "status",
            "designation", "job", "title", "role", "work", "count",
            "engineering", "it", "hr", "finance", "sales", "operations",
            "nonexistent"
        ];

        for pat in patterns {
            if let Ok(re) = Regex::new(pat) {
                if let Some(caps) = re.captures(text) {
                    if let Some(m) = caps.get(1) {
                        let candidate = m.as_str();
                        let lower = candidate.to_lowercase();
                        if !stop_words.contains(&lower.as_str()) {
                            // Also ensure it is not matching a known designation, department, or status
                            if self.matcher.resolve_department(candidate).is_none() 
                               && self.matcher.resolve_designation(candidate).is_none()
                               && self.matcher.resolve_status(candidate).is_none() {
                                return Some(candidate.to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

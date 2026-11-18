use crate::pipeline_models::{ContextEmployee, ConversationContext};
use regex::Regex;
use serde_json::Value;

pub enum ReferenceResolutionResult {
    /// Resolved to a single employee ID with a specific field request (e.g. "What is his phone number?")
    FieldQuery {
        employee_id: i32,
        resolved_label: String,
        target_field: String,
    },
    /// Resolved to multiple employees for a plural property request (e.g. "What are their phone numbers?")
    CollectionFieldQuery {
        employee_ids: Vec<i32>,
        resolved_label: String,
        target_field: String,
    },
    /// Resolved to an update operation with the target employee ID substituted (e.g. "Update his phone number to 0771231234")
    UpdateAction {
        employee_id: i32,
        resolved_label: String,
        modified_prompt: String,
    },
    /// Ambiguous pronoun (e.g. "her" when multiple female employees or multiple employees exist)
    Ambiguous {
        message: String,
        options: Vec<String>,
    },
    /// Pronoun detected but no prior conversation context exists
    NoContext {
        message: String,
    },
    /// No pronoun or reference was detected in the prompt
    NotApplicable,
}

pub struct ReferenceResolver;

impl ReferenceResolver {
    /// Inferred gender helper for known first names in InternAgenticDB
    pub fn infer_gender(first_name: &str) -> Option<&'static str> {
        let lower = first_name.to_lowercase();
        match lower.as_str() {
            "amanda" | "ashley" | "emily" | "jennifer" | "jessica" | "linda" | "nicole" | "sarah" | "sophia" => {
                Some("female")
            }
            "andrew" | "christopher" | "daniel" | "david" | "james" | "john" | "matthew" | "michael" | "robert" | "ryan" | "william" => {
                Some("male")
            }
            _ => None,
        }
    }

    /// Try resolving an anaphoric reference (pronoun or noun phrase) using the session context
    pub fn resolve(prompt: &str, context: &Option<ConversationContext>) -> ReferenceResolutionResult {
        let trimmed = prompt.trim();

        // 1. Check for pronoun / referential phrase
        let pronoun_re = Regex::new(
            r"(?i)\b(he|his|him|she|her|they|their|them|that employee|this employee|the previous employee|that person|this person)\b"
        ).unwrap();

        let caps = match pronoun_re.captures(trimmed) {
            Some(c) => c,
            None => return ReferenceResolutionResult::NotApplicable,
        };

        let pronoun = caps.get(1).unwrap().as_str().to_lowercase();

        // 2. If no context or empty context, return NoContext
        let ctx = match context {
            Some(c) if c.last_referenced_employee_id.is_some() || !c.last_employees_collection.is_empty() => c,
            _ => {
                return ReferenceResolutionResult::NoContext {
                    message: format!(
                        "You referenced '{}', but there is no previous employee in this conversation session. Please specify the employee by ID or name.",
                        caps.get(1).unwrap().as_str()
                    ),
                };
            }
        };

        // 3. Check if this is an update action (e.g. "Update his phone number to 0771231234" or "change her salary to 90000")
        let update_verb_re = Regex::new(r"(?i)\b(update|change|set|modify)\b").unwrap();
        if update_verb_re.is_match(trimmed) {
            // Pick target employee ID
            let target_emp = match Self::pick_employee(&pronoun, ctx) {
                Ok(emp) => emp,
                Err(res) => return res,
            };

            // Replace pronoun with "employee ID <id>"
            let replaced = pronoun_re.replace(trimmed, format!("employee ID {}", target_emp.employee_id)).to_string();

            return ReferenceResolutionResult::UpdateAction {
                employee_id: target_emp.employee_id,
                resolved_label: format!("{} → {} (Employee ID {})", caps.get(1).unwrap().as_str(), target_emp.full_name, target_emp.employee_id),
                modified_prompt: replaced,
            };
        }

        // 4. Check for field inquiry (e.g. "What is his phone number?", "Which department does he work in?", "What is his salary?")
        let field_re = Regex::new(
            r"(?i)\b(hire(?:d|\s+dates?)?|start\s+dates?|join(?:ed)?|phone(?:\s*numbers?|\s*number)?|emails?(?:\s*address(?:es)?)?|departments?|designations?|titles?|roles?|salar(?:y|ies)|pay|compensation|names?|status(?:es)?)\b"
        ).unwrap();

        let target_field = if let Some(fc) = field_re.captures(trimmed) {
            let f = fc.get(1).unwrap().as_str().to_lowercase();
            if f.starts_with("hire") || f.starts_with("start") || f.starts_with("join") {
                "hire_date"
            } else if f.starts_with("phone") {
                "phone"
            } else if f.starts_with("email") {
                "email"
            } else if f.starts_with("dept") || f.starts_with("department") {
                "department"
            } else if f.starts_with("desig") || f.starts_with("title") || f.starts_with("role") {
                "designation"
            } else if f.starts_with("salar") || f == "pay" || f == "compensation" {
                "salary"
            } else if f.starts_with("name") {
                "name"
            } else if f.starts_with("status") {
                "status"
            } else {
                "details"
            }
        } else {
            "details"
        };

        // 5. Handle Plural Reference ("they", "their", "them")
        if pronoun == "they" || pronoun == "their" || pronoun == "them" {
            let ids: Vec<i32> = if !ctx.last_employees_collection.is_empty() {
                ctx.last_employees_collection.iter().map(|e| e.employee_id).collect()
            } else if let Some(single) = ctx.last_referenced_employee_id {
                vec![single]
            } else {
                vec![]
            };

            if ids.is_empty() {
                return ReferenceResolutionResult::NoContext {
                    message: "There are no employees from the previous search to reference.".to_string(),
                };
            }

            return ReferenceResolutionResult::CollectionFieldQuery {
                employee_ids: ids,
                resolved_label: format!("{} → {} employee(s) from previous result", caps.get(1).unwrap().as_str(), ctx.last_employees_collection.len()),
                target_field: target_field.to_string(),
            };
        }

        // 6. Handle Singular Reference ("he", "his", "she", "her", "that employee", etc.)
        let target_emp = match Self::pick_employee(&pronoun, ctx) {
            Ok(emp) => emp,
            Err(res) => return res,
        };

        ReferenceResolutionResult::FieldQuery {
            employee_id: target_emp.employee_id,
            resolved_label: format!("{} → {} (Employee ID {})", caps.get(1).unwrap().as_str(), target_emp.full_name, target_emp.employee_id),
            target_field: target_field.to_string(),
        }
    }

    fn pick_employee(pronoun: &str, ctx: &ConversationContext) -> Result<ContextEmployee, ReferenceResolutionResult> {
        let is_female = pronoun == "she" || pronoun == "her";
        let is_male = pronoun == "he" || pronoun == "his" || pronoun == "him";

        // If collection has multiple items:
        if ctx.last_employees_collection.len() > 1 {
            if is_female {
                let females: Vec<_> = ctx.last_employees_collection.iter()
                    .filter(|e| e.gender.as_deref() == Some("female"))
                    .cloned()
                    .collect();
                if females.len() == 1 {
                    return Ok(females[0].clone());
                } else if females.len() > 1 {
                    let names: Vec<String> = females.iter().map(|e| format!("{} (ID: {})", e.full_name, e.employee_id)).collect();
                    return Err(ReferenceResolutionResult::Ambiguous {
                        message: format!("The previous result contained multiple female employees: {}. Which employee would you like to inspect?", names.join(", ")),
                        options: names,
                    });
                }
            } else if is_male {
                let males: Vec<_> = ctx.last_employees_collection.iter()
                    .filter(|e| e.gender.as_deref() == Some("male"))
                    .cloned()
                    .collect();
                if males.len() == 1 {
                    return Ok(males[0].clone());
                } else if males.len() > 1 {
                    let names: Vec<String> = males.iter().map(|e| format!("{} (ID: {})", e.full_name, e.employee_id)).collect();
                    return Err(ReferenceResolutionResult::Ambiguous {
                        message: format!("The previous result contained multiple male employees: {}. Which employee would you like to inspect?", names.join(", ")),
                        options: names,
                    });
                }
            }

            // Ambiguous gender or neutral "that employee" with multiple employees
            let all_names: Vec<String> = ctx.last_employees_collection.iter().map(|e| format!("{} (ID: {})", e.full_name, e.employee_id)).collect();
            return Err(ReferenceResolutionResult::Ambiguous {
                message: format!("The previous result returned multiple employees: {}. Please specify which employee you are asking about.", all_names.join(", ")),
                options: all_names,
            });
        }

        // Single employee in collection or last_referenced_employee
        if let Some(emp) = &ctx.last_referenced_employee {
            return Ok(emp.clone());
        }
        if let Some(first) = ctx.last_employees_collection.first() {
            return Ok(first.clone());
        }

        Err(ReferenceResolutionResult::NoContext {
            message: "No employee record is currently referenced in this conversation.".to_string(),
        })
    }

    /// Extract employee metadata from tool execution result to update conversation context
    pub fn extract_employees_from_result(_tool_name: &str, result: &Value) -> Vec<ContextEmployee> {
        let mut list = Vec::new();

        let data = result.get("data").unwrap_or(result);

        if data.is_array() {
            if let Some(arr) = data.as_array() {
                for item in arr {
                    if let Some(emp) = Self::json_to_context_employee(item) {
                        list.push(emp);
                    }
                }
            }
        } else if data.is_object() {
            if let Some(emp) = Self::json_to_context_employee(data) {
                list.push(emp);
            }
        }

        list
    }

    fn json_to_context_employee(val: &Value) -> Option<ContextEmployee> {
        let id = val.get("EmployeeID").and_then(|v| v.as_i64()).map(|n| n as i32)?;
        let first = val.get("FirstName").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let last = val.get("LastName").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let full = format!("{} {}", first, last).trim().to_string();
        let dept = val.get("DepartmentName").and_then(|v| v.as_str()).map(|s| s.to_string());
        let desig = val.get("DesignationName").and_then(|v| v.as_str()).map(|s| s.to_string());
        let email = val.get("Email").and_then(|v| v.as_str()).map(|s| s.to_string());
        let phone = val.get("Phone").and_then(|v| v.as_str()).map(|s| s.to_string());
        let salary = val.get("Salary").and_then(|v| v.as_f64());
        let status = val.get("Status").and_then(|v| v.as_str()).map(|s| s.to_string());
        let gender = Self::infer_gender(&first).map(|s| s.to_string());

        Some(ContextEmployee {
            employee_id: id,
            first_name: first,
            last_name: last,
            full_name: full,
            department: dept,
            designation: desig,
            email,
            phone,
            salary,
            status,
            gender,
        })
    }
}

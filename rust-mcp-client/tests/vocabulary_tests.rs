use rust_mcp_client::{VocabularyLoader, VocabularyMatcher};

#[test]
fn test_vocabulary_load_embedded() {
    let vocab = VocabularyLoader::load_embedded().expect("Should load embedded vocabulary.json");
    assert_eq!(vocab.version, "1.0");
    assert_eq!(vocab.intents.len(), 7);

    let intent_tools: Vec<String> = vocab.intents.iter().map(|i| i.target_tool.clone()).collect();
    assert!(intent_tools.contains(&"get_all_employees".to_string()));
    assert!(intent_tools.contains(&"get_employee_by_id".to_string()));
    assert!(intent_tools.contains(&"search_employees".to_string()));
    assert!(intent_tools.contains(&"get_department_list".to_string()));
    assert!(intent_tools.contains(&"get_employee_count".to_string()));
    assert!(intent_tools.contains(&"add_employee".to_string()));
    assert!(intent_tools.contains(&"update_employee".to_string()));
}

#[test]
fn test_case_insensitive_matching() {
    let vocab = VocabularyLoader::load_embedded().unwrap();
    let matcher = VocabularyMatcher::new(vocab.clone()).unwrap();

    assert!(matcher.matches_intent("get_all_employees", "SHOW ALL EMPLOYEES"));
    assert!(matcher.matches_intent("get_all_employees", "list all employees"));
    assert!(matcher.matches_intent("get_all_employees", "Display All Staff"));
    assert!(matcher.matches_intent("get_all_employees", "Fetch employee roster"));
    assert!(matcher.matches_intent("get_department_list", "show all departments"));
    assert!(matcher.matches_intent("get_department_list", "LIST DIVISIONS"));
    assert!(matcher.matches_intent("get_employee_count", "how many employees are there"));
    assert!(matcher.matches_intent("get_employee_count", "count staff"));
}

#[test]
fn test_word_boundaries_and_aliases() {
    let vocab = VocabularyLoader::load_embedded().unwrap();
    let matcher = VocabularyMatcher::new(vocab.clone()).unwrap();

    // Department alias resolution
    assert_eq!(matcher.resolve_department("Show employees in IT department"), Some("IT".to_string()));
    assert_eq!(matcher.resolve_department("Find workers in tech"), Some("IT".to_string()));
    assert_eq!(matcher.resolve_department("List HR team"), Some("HR".to_string()));
    assert_eq!(matcher.resolve_department("Accounting staff"), Some("Finance".to_string()));

    // Ensure word boundary prevents false substring matches (e.g., "hit" should not match "it")
    assert_eq!(matcher.resolve_department("They hit a milestone"), None);

    // Designation alias resolution
    assert_eq!(matcher.resolve_designation("Find senior dev"), Some("Senior Software Engineer".to_string()));
    assert_eq!(matcher.resolve_designation("Show QA engineer"), Some("QA Engineer".to_string()));
    assert_eq!(matcher.resolve_designation("Lookup business analyst"), Some("Business Analyst".to_string()));

    // Status alias resolution
    assert_eq!(matcher.resolve_status("Employees on leave"), Some("On Leave".to_string()));
    assert_eq!(matcher.resolve_status("Find active personnel"), Some("Active".to_string()));
    assert_eq!(matcher.resolve_status("Show inactive workers"), Some("Inactive".to_string()));
}

#[test]
fn test_parameter_extraction() {
    let vocab = VocabularyLoader::load_embedded().unwrap();
    let matcher = VocabularyMatcher::new(vocab.clone()).unwrap();

    assert_eq!(matcher.extract_employee_id("Find employee ID 1001"), Some(1001));
    assert_eq!(matcher.extract_employee_id("get employee #1020 details"), Some(1020));
    assert_eq!(matcher.extract_employee_id("show profile 1005"), Some(1005));

    assert_eq!(
        matcher.extract_email("Register new user with email john.doe@spillabs.com please"),
        Some("john.doe@spillabs.com".to_string())
    );
}

#[test]
fn test_write_intent_identification() {
    let vocab = VocabularyLoader::load_embedded().unwrap();
    let matcher = VocabularyMatcher::new(vocab.clone()).unwrap();

    assert!(matcher.matches_intent("add_employee", "Add a new employee named Alice"));
    assert!(matcher.matches_intent("add_employee", "Create employee Bob"));
    assert!(matcher.matches_intent("add_employee", "Register new hire Charlie"));

    assert!(matcher.matches_intent("update_employee", "Update employee 1001 department to IT"));
    assert!(matcher.matches_intent("update_employee", "Modify employee 1002 salary"));
    assert!(matcher.matches_intent("update_employee", "Change employee status"));

    // Verify is_write flag is accurately set
    let add_intent = vocab.intents.iter().find(|i| i.intent_id == "add_employee").unwrap();
    let upd_intent = vocab.intents.iter().find(|i| i.intent_id == "update_employee").unwrap();
    let get_intent = vocab.intents.iter().find(|i| i.intent_id == "get_all_employees").unwrap();

    assert!(add_intent.is_write);
    assert!(upd_intent.is_write);
    assert!(!get_intent.is_write);
}

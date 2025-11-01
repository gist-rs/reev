use reev_lib::parsing::deterministic_parser::DeterministicParser;

#[test]
fn test_is_deterministic_response() {
    let deterministic_response = r#"
        {
            "result": {"text": [{"program_id":"11111111111111111111111111111111"}]},
            "transactions": null,
            "summary": null
        }
        "#;

    assert!(DeterministicParser::is_deterministic_response(
        deterministic_response
    ));

    let non_deterministic_response = r#"
        {
            "transactions": [{"program_id":"11111111111111111111111111111111"}],
            "result": null
        }
        "#;

    assert!(!DeterministicParser::is_deterministic_response(
        non_deterministic_response
    ));
}

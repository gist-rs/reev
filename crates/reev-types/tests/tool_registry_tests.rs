//! Tests for tool registry module

use reev_types::tool_registry::ToolRegistry;

#[test]
fn test_all_tools_exist() {
    let tools = ToolRegistry::all_tools();
    assert!(!tools.is_empty());

    // Verify all tools are valid
    for tool in &tools {
        assert!(ToolRegistry::is_valid_tool(tool.as_str()));
    }
}

#[test]
fn test_tool_categories() {
    let jupiter_tools = ToolRegistry::jupiter_tools();
    assert!(!jupiter_tools.is_empty());

    // All Jupiter tools should be valid
    for tool in &jupiter_tools {
        assert!(ToolRegistry::is_valid_tool(tool.as_str()));
    }
}

#[test]
fn test_category_separation() {
    let discovery_tools = ToolRegistry::discovery_tools();
    let swap_tools = ToolRegistry::swap_tools();
    let lending_tools = ToolRegistry::lending_tools();
    let position_tools = ToolRegistry::position_tools();

    // Verify categories are non-overlapping
    let all_discovery: std::collections::HashSet<_> =
        discovery_tools.iter().map(|s| s.as_str()).collect();
    let all_swap: std::collections::HashSet<_> = swap_tools.iter().map(|s| s.as_str()).collect();
    let all_lending: std::collections::HashSet<_> =
        lending_tools.iter().map(|s| s.as_str()).collect();
    let all_position: std::collections::HashSet<_> =
        position_tools.iter().map(|s| s.as_str()).collect();

    // Verify we have expected number of tools
    assert_eq!(all_discovery.len(), 2);
    assert_eq!(all_swap.len(), 4);
    assert_eq!(all_lending.len(), 4);
    assert_eq!(all_position.len(), 1);

    // Tools should only appear in one category
    assert!(all_discovery
        .intersection(&all_swap)
        .collect::<Vec<_>>()
        .is_empty());
    assert!(all_discovery
        .intersection(&all_lending)
        .collect::<Vec<_>>()
        .is_empty());
    assert!(all_discovery
        .intersection(&all_position)
        .collect::<Vec<_>>()
        .is_empty());
}

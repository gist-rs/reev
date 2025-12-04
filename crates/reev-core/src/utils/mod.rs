//! Utility modules for YML prompt creation, result processing, and execution flow
//!
//! This module provides utilities for creating standardized YML prompts,
//! processing execution results, and executing standardized flows.
//! These utilities are designed to be used across tests, API, and runner components.

pub mod flow_utils;
pub mod result_utils;
pub mod transfer_utils;
pub mod wallet_utils;
pub mod yml_utils;

// Re-export key utilities for convenience
pub use flow_utils::{
    create_flow_from_prompt, create_standardized_test_result, create_yml_prompt_for_flow,
    determine_operation_type, execute_six_step_flow, extract_recipient_from_prompt,
    validate_execution_result,
};
pub use result_utils::{
    extract_error_messages, extract_metadata, extract_tool_results, extract_transaction_signature,
    is_execution_successful,
};
pub use transfer_utils::calculate_max_transferable_amount;
pub use wallet_utils::create_wallet_context;
pub use yml_utils::{
    create_complete_yml_with_ground_truth, create_ground_truth_yml, create_lend_step,
    create_subject_wallet_info_yml, create_swap_step, create_transfer_step, create_yml_prompt,
    parse_yml, validate_yml_structure, YmlAssertionData, YmlStepData, YmlToolCallData,
};

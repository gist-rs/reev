//! Step-Specific Constraints for Enhanced Context Passing
//!
//! This module implements step-specific constraints that can be applied
//! to guide tool selection and parameter validation in multi-step flows.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
// HashMap is used through Value::Object, but not directly

/// Types of constraints that can be applied to operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Maximum amount for a parameter
    MaximumAmount(f64),
    /// Minimum amount for a parameter
    MinimumAmount(f64),
    /// Required mint address
    RequiredMint(String),
    /// Excluded mint address
    ExcludedMint(String),
    /// Maximum price slippage percentage
    PriceSlippage(f64),
    /// Minimum liquidity required
    MinimumLiquidity(f64),
    /// Time constraint
    TimeLimit(chrono::DateTime<chrono::Utc>),
    /// Custom constraint with parameters
    Custom(String, Value),
}

impl ConstraintType {
    /// Check if a value satisfies this constraint
    pub fn validate_value(&self, value: &Value) -> Result<bool> {
        match self {
            ConstraintType::MaximumAmount(max) => {
                if let Some(amount) = value.as_f64() {
                    Ok(amount <= *max)
                } else if let Some(amount) = value.as_u64() {
                    Ok(amount as f64 <= *max)
                } else {
                    Ok(false) // Invalid type for amount comparison
                }
            }
            ConstraintType::MinimumAmount(min) => {
                if let Some(amount) = value.as_f64() {
                    Ok(amount >= *min)
                } else if let Some(amount) = value.as_u64() {
                    Ok(amount as f64 >= *min)
                } else {
                    Ok(false) // Invalid type for amount comparison
                }
            }
            ConstraintType::RequiredMint(required_mint) => {
                if let Some(mint) = value.as_str() {
                    Ok(mint == required_mint)
                } else {
                    Ok(false)
                }
            }
            ConstraintType::ExcludedMint(excluded_mint) => {
                if let Some(mint) = value.as_str() {
                    Ok(mint != excluded_mint)
                } else {
                    Ok(true) // Non-string values are not excluded mints
                }
            }
            ConstraintType::PriceSlippage(max_slippage) => {
                if let Some(slippage) = value.as_f64() {
                    Ok(slippage <= *max_slippage)
                } else {
                    Ok(false)
                }
            }
            ConstraintType::MinimumLiquidity(min_liquidity) => {
                if let Some(liquidity) = value.as_f64() {
                    Ok(liquidity >= *min_liquidity)
                } else {
                    Ok(false)
                }
            }
            ConstraintType::TimeLimit(deadline) => {
                // For time constraints, we just check if current time is before deadline
                let now = chrono::Utc::now();
                Ok(now < *deadline)
            }
            ConstraintType::Custom(_name, _params) => {
                // Custom constraints would need specific validation logic
                // For now, we'll default to true
                Ok(true)
            }
        }
    }

    /// Get a human-readable description of the constraint
    pub fn description(&self) -> String {
        match self {
            ConstraintType::MaximumAmount(max) => format!("Amount must be <= {max}"),
            ConstraintType::MinimumAmount(min) => format!("Amount must be >= {min}"),
            ConstraintType::RequiredMint(mint) => format!("Must use mint: {mint}"),
            ConstraintType::ExcludedMint(mint) => format!("Cannot use mint: {mint}"),
            ConstraintType::PriceSlippage(slippage) => {
                format!("Slippage must be <= {}%", slippage * 100.0)
            }
            ConstraintType::MinimumLiquidity(liquidity) => {
                format!("Liquidity must be >= {liquidity}")
            }
            ConstraintType::TimeLimit(deadline) => format!(
                "Must complete before {}",
                deadline.format("%Y-%m-%d %H:%M:%S UTC")
            ),
            ConstraintType::Custom(name, _) => format!("Custom constraint: {name}"),
        }
    }
}

/// A step-specific constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConstraint {
    /// Type of constraint
    pub constraint_type: ConstraintType,
    /// Parameter name this constraint applies to (optional)
    pub parameter_name: Option<String>,
    /// Tool name this constraint applies to (optional)
    pub tool_name: Option<String>,
    /// Steps this constraint applies to (empty means all steps)
    pub applicable_steps: Vec<usize>,
    /// Whether this constraint is mandatory
    pub mandatory: bool,
}

impl StepConstraint {
    /// Create a new constraint
    pub fn new(constraint_type: ConstraintType) -> Self {
        Self {
            constraint_type,
            parameter_name: None,
            tool_name: None,
            applicable_steps: Vec::new(),
            mandatory: true,
        }
    }

    /// Set the parameter name this constraint applies to
    pub fn for_parameter(mut self, parameter: &str) -> Self {
        self.parameter_name = Some(parameter.to_string());
        self
    }

    /// Set the tool name this constraint applies to
    pub fn for_tool(mut self, tool: &str) -> Self {
        self.tool_name = Some(tool.to_string());
        self
    }

    /// Set the applicable steps
    pub fn for_steps(mut self, steps: Vec<usize>) -> Self {
        self.applicable_steps = steps;
        self
    }

    /// Set whether this constraint is mandatory
    pub fn mandatory(mut self, mandatory: bool) -> Self {
        self.mandatory = mandatory;
        self
    }

    /// Check if this constraint applies to a specific step
    pub fn applies_to_step(&self, step_number: usize) -> bool {
        self.applicable_steps.is_empty() || self.applicable_steps.contains(&step_number)
    }

    /// Check if this constraint applies to a specific tool
    pub fn applies_to_tool(&self, tool_name: &str) -> bool {
        match &self.tool_name {
            Some(t) => t == tool_name,
            None => true, // Applies to all tools if not specified
        }
    }

    /// Validate a tool against this constraint
    pub fn validate_tool_parameters(
        &self,
        tool_name: &str,
        parameters: &Value,
    ) -> Result<ConstraintValidationResult> {
        // Check if constraint applies to this tool
        if !self.applies_to_tool(tool_name) {
            return Ok(ConstraintValidationResult::NotApplicable);
        }

        // If parameter_name is specified, validate only that parameter
        if let Some(param_name) = &self.parameter_name {
            if let Some(param_value) = parameters.get(param_name) {
                let is_valid = self.constraint_type.validate_value(param_value)?;
                if is_valid {
                    Ok(ConstraintValidationResult::Valid)
                } else {
                    Ok(ConstraintValidationResult::Invalid {
                        constraint: self.constraint_type.description(),
                        parameter: param_name.clone(),
                        value: param_value.clone(),
                        mandatory: self.mandatory,
                    })
                }
            } else {
                // Parameter not found
                if self.mandatory {
                    Ok(ConstraintValidationResult::Invalid {
                        constraint: self.constraint_type.description(),
                        parameter: param_name.clone(),
                        value: Value::Null,
                        mandatory: self.mandatory,
                    })
                } else {
                    Ok(ConstraintValidationResult::NotApplicable)
                }
            }
        } else {
            // No specific parameter, check if any parameter violates the constraint
            if let Some(obj) = parameters.as_object() {
                for (param_name, param_value) in obj {
                    let is_valid = self.constraint_type.validate_value(param_value)?;
                    if !is_valid {
                        return Ok(ConstraintValidationResult::Invalid {
                            constraint: self.constraint_type.description(),
                            parameter: param_name.clone(),
                            value: param_value.clone(),
                            mandatory: self.mandatory,
                        });
                    }
                }
                Ok(ConstraintValidationResult::Valid)
            } else {
                // Parameters is not an object, can't validate
                Ok(ConstraintValidationResult::NotApplicable)
            }
        }
    }
}

/// Result of constraint validation
#[derive(Debug, Clone)]
pub enum ConstraintValidationResult {
    /// Constraint is satisfied
    Valid,
    /// Constraint is violated
    Invalid {
        constraint: String,
        parameter: String,
        value: Value,
        mandatory: bool,
    },
    /// Constraint doesn't apply to this context
    NotApplicable,
}

/// Builder for creating constraints
pub struct ConstraintBuilder {
    constraints: Vec<StepConstraint>,
}

impl ConstraintBuilder {
    /// Create a new constraint builder
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Add a maximum amount constraint
    pub fn max_amount(&mut self, amount: f64, parameter: Option<&str>) -> &mut Self {
        let constraint = StepConstraint::new(ConstraintType::MaximumAmount(amount));
        let constraint = if let Some(param) = parameter {
            constraint.for_parameter(param)
        } else {
            constraint
        };
        self.constraints.push(constraint);
        self
    }

    /// Add a minimum amount constraint
    pub fn min_amount(&mut self, amount: f64, parameter: Option<&str>) -> &mut Self {
        let constraint = StepConstraint::new(ConstraintType::MinimumAmount(amount));
        let constraint = if let Some(param) = parameter {
            constraint.for_parameter(param)
        } else {
            constraint
        };
        self.constraints.push(constraint);
        self
    }

    /// Add a required mint constraint
    pub fn required_mint(&mut self, mint: &str, parameter: Option<&str>) -> &mut Self {
        let constraint = StepConstraint::new(ConstraintType::RequiredMint(mint.to_string()));
        let constraint = if let Some(param) = parameter {
            constraint.for_parameter(param)
        } else {
            constraint
        };
        self.constraints.push(constraint);
        self
    }

    /// Add an excluded mint constraint
    pub fn excluded_mint(&mut self, mint: &str, parameter: Option<&str>) -> &mut Self {
        let constraint = StepConstraint::new(ConstraintType::ExcludedMint(mint.to_string()));
        let constraint = if let Some(param) = parameter {
            constraint.for_parameter(param)
        } else {
            constraint
        };
        self.constraints.push(constraint);
        self
    }

    /// Add a maximum slippage constraint
    pub fn max_slippage(&mut self, slippage: f64, parameter: Option<&str>) -> &mut Self {
        let constraint = StepConstraint::new(ConstraintType::PriceSlippage(slippage));
        let constraint = if let Some(param) = parameter {
            constraint.for_parameter(param)
        } else {
            constraint
        };
        self.constraints.push(constraint);
        self
    }

    /// Add a custom constraint
    pub fn custom(&mut self, name: &str, params: Value, parameter: Option<&str>) -> &mut Self {
        let constraint = StepConstraint::new(ConstraintType::Custom(name.to_string(), params));
        let constraint = if let Some(param) = parameter {
            constraint.for_parameter(param)
        } else {
            constraint
        };
        self.constraints.push(constraint);
        self
    }

    /// Build the constraints
    pub fn build(self) -> Vec<StepConstraint> {
        self.constraints
    }
}

impl Default for ConstraintBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Validator for tool parameters against constraints
pub struct ParameterValidator {
    constraints: Vec<StepConstraint>,
}

impl ParameterValidator {
    /// Create a new parameter validator
    pub fn new(constraints: Vec<StepConstraint>) -> Self {
        Self { constraints }
    }

    /// Validate tool parameters against all constraints
    pub fn validate(
        &self,
        tool_name: &str,
        parameters: &Value,
        step_number: usize,
    ) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        for constraint in &self.constraints {
            // Check if constraint applies to this step
            if !constraint.applies_to_step(step_number) {
                continue;
            }

            // Validate against this constraint
            match constraint.validate_tool_parameters(tool_name, parameters)? {
                ConstraintValidationResult::Valid => {
                    report.add_satisfied_constraint(constraint);
                }
                ConstraintValidationResult::Invalid {
                    constraint,
                    parameter,
                    value,
                    mandatory,
                } => {
                    if mandatory {
                        report.add_mandatory_violation(constraint, parameter, value);
                    } else {
                        report.add_optional_violation(constraint, parameter, value);
                    }
                }
                ConstraintValidationResult::NotApplicable => {
                    // Constraint doesn't apply to this context
                    report.add_not_applicable_constraint(constraint);
                }
            }
        }

        Ok(report)
    }
}

/// Report of validation results
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Constraints that were satisfied
    pub satisfied: Vec<String>,
    /// Mandatory constraints that were violated
    pub mandatory_violations: Vec<Violation>,
    /// Optional constraints that were violated
    pub optional_violations: Vec<Violation>,
    /// Constraints that didn't apply
    pub not_applicable: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Violation {
    pub constraint: String,
    pub parameter: String,
    pub value: Value,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new() -> Self {
        Self {
            satisfied: Vec::new(),
            mandatory_violations: Vec::new(),
            optional_violations: Vec::new(),
            not_applicable: Vec::new(),
        }
    }

    /// Add a satisfied constraint
    fn add_satisfied_constraint(&mut self, constraint: &StepConstraint) {
        self.satisfied
            .push(constraint.constraint_type.description());
    }

    /// Add a mandatory constraint violation
    fn add_mandatory_violation(&mut self, constraint: String, parameter: String, value: Value) {
        self.mandatory_violations.push(Violation {
            constraint,
            parameter,
            value,
        });
    }

    /// Add an optional constraint violation
    fn add_optional_violation(&mut self, constraint: String, parameter: String, value: Value) {
        self.optional_violations.push(Violation {
            constraint,
            parameter,
            value,
        });
    }

    /// Add a not applicable constraint
    fn add_not_applicable_constraint(&mut self, constraint: &StepConstraint) {
        self.not_applicable
            .push(constraint.constraint_type.description());
    }

    /// Check if validation passed (no mandatory violations)
    pub fn is_valid(&self) -> bool {
        self.mandatory_violations.is_empty()
    }

    /// Get all violations
    pub fn all_violations(&self) -> Vec<&Violation> {
        let mut violations = Vec::new();
        violations.extend(&self.mandatory_violations);
        violations.extend(&self.optional_violations);
        violations
    }

    /// Get a summary of the validation report
    pub fn summary(&self) -> String {
        if self.is_valid() {
            format!(
                "Validation passed: {} constraints satisfied, {} optional violations",
                self.satisfied.len(),
                self.optional_violations.len()
            )
        } else {
            format!(
                "Validation failed: {} mandatory violations, {} satisfied",
                self.mandatory_violations.len(),
                self.satisfied.len()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_constraint_validation() {
        let mut builder = ConstraintBuilder::new();
        builder.max_amount(1000.0, Some("amount"));
        let constraints = builder.build();

        let validator = ParameterValidator::new(constraints);

        // Test valid parameters
        let params = json!({"amount": 500});
        let report = validator.validate("jupiter_swap", &params, 0).unwrap();
        assert!(report.is_valid());

        // Test invalid parameters
        let params = json!({"amount": 1500});
        let report = validator.validate("jupiter_swap", &params, 0).unwrap();
        assert!(!report.is_valid());
    }

    #[test]
    fn test_mint_constraint() {
        let mut builder = ConstraintBuilder::new();
        builder.required_mint("mint123", Some("input_mint"));
        let constraints = builder.build();

        let validator = ParameterValidator::new(constraints);

        // Test valid parameters
        let params = json!({"input_mint": "mint123"});
        let report = validator.validate("jupiter_swap", &params, 0).unwrap();
        assert!(report.is_valid());

        // Test invalid parameters
        let params = json!({"input_mint": "mint456"});
        let report = validator.validate("jupiter_swap", &params, 0).unwrap();
        assert!(!report.is_valid());
    }
}

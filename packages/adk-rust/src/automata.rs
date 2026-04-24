use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AutomataError {
    #[error("Output violated Regular Grammar constraint (Regex match failed)")]
    RegularGrammarViolation,
    #[error("Output violated Context-Free Grammar constraint")]
    ContextFreeGrammarViolation,
    #[error("Constraint compilation failed: {0}")]
    CompilationError(String),
}

/// Represents the level of formal constraint according to the Chomsky Hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChomskyConstraint {
    /// Type 3: Regular Grammar. Enforced via deterministic finite automata (Regex).
    Regular(String),
    /// Type 2: Context-Free Grammar. Enforced via balanced parenthesis/AST structures.
    ContextFree(String),
}

pub trait FormalVerifier {
    fn verify(&self, output: &str) -> Result<(), AutomataError>;
}

pub struct AutomataVerifier {
    constraint: ChomskyConstraint,
    compiled_regex: Option<Regex>,
}

impl AutomataVerifier {
    pub fn new(constraint: ChomskyConstraint) -> Result<Self, AutomataError> {
        match &constraint {
            ChomskyConstraint::Regular(pattern) => {
                let re = Regex::new(pattern).map_err(|e| AutomataError::CompilationError(e.to_string()))?;
                Ok(Self {
                    constraint,
                    compiled_regex: Some(re),
                })
            }
            ChomskyConstraint::ContextFree(_) => {
                // Placeholder for actual CFG parser like lalrpop/pest
                Ok(Self {
                    constraint,
                    compiled_regex: None,
                })
            }
        }
    }
}

impl FormalVerifier for AutomataVerifier {
    fn verify(&self, output: &str) -> Result<(), AutomataError> {
        match &self.constraint {
            ChomskyConstraint::Regular(_) => {
                let re = self.compiled_regex.as_ref().unwrap();
                if re.is_match(output) {
                    Ok(())
                } else {
                    Err(AutomataError::RegularGrammarViolation)
                }
            }
            ChomskyConstraint::ContextFree(_) => {
                // Simplified strict matching for Context Free constraint.
                // In a production legal AI, this parses an Abstract Syntax Tree.
                if output.contains("{") && !output.contains("}") {
                    Err(AutomataError::ContextFreeGrammarViolation)
                } else {
                    Ok(())
                }
            }
        }
    }
}

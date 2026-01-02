// v0.0.597: Settings Validator Chain Module
// Chainable validation pipeline for settings

mod chain;
mod error;
mod manager;
mod types;
mod utils;
mod validator;

// Re-export public API to preserve compatibility
pub use chain::{ChainResult, ValidationChain};
pub use error::ValidationError;
pub use manager::ValidatorChainManager;
pub use types::{ValidationResult, ValidatorType};
pub use utils::{format_validator_chain, is_validator_chain_query, validator_chain_fun_fact};
pub use validator::{ValidatorDef, ValidationOutput};

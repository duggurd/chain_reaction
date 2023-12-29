use crate::errors::ExecError;

use super::OperatorT;

#[derive(PartialEq, Debug, Eq)]
// Compiles and executes rust source code
pub struct RustOperator {
}

impl OperatorT for RustOperator {
    fn execute(&self) -> Result<(), crate::errors::ExecError> {
        Err(ExecError{message: "Not implemented".to_string()})
    }
}
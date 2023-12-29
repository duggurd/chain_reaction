use crate::{operators::OperatorT, errors::ExecError};

#[derive(PartialEq, Debug, Eq)]
// Uses python bindings?
pub struct PythonOperator {
    code: String
}

impl OperatorT for PythonOperator {
    fn execute(&self) -> Result<(), crate::errors::ExecError> {
        Err(ExecError{message: "Not implemented".to_string()})
    }
}

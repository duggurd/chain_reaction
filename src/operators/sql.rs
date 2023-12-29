use crate::errors::ExecError;
use super::OperatorT;

#[derive(PartialEq, Debug, Eq)]
pub struct SQLOperator {
    connection: (),
    sql: String
}

impl OperatorT for SQLOperator {
    fn execute(&self) -> Result<(), crate::errors::ExecError> {
        Err(ExecError{message: "Not implemented".to_string()})
    }
}
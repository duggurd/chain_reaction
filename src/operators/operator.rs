use log::error;

use crate::errors::ExecError;
use crate::operators::*;

// Finding all rust funttions 
// regexp_find_all("fn[ ]+(\w+)[ ]?(")

#[derive(PartialEq, Eq, Debug)]
pub enum Operator {
    #[cfg(feature="rust")]
    RustOp(RustOperator),

    #[cfg(feature="python")]
    PythonOp(PythonOperator),

    #[cfg(feature="shell")]
    ShellOp(ShellOperator)
}

impl Operator {
    pub(crate) fn execute(&self) -> Result<(), ExecError> {
        match self {
            #[cfg(feature="python")]
            Operator::PythonOp(inner) => {inner.execute()?}
            
            #[cfg(feature="rust")]
            Operator::RustOp(inner) => {inner.execute()?}
            
            #[cfg(feature="shell")]
            Operator::ShellOp(inner) => {inner.execute()?}

            s => {error!("No operator matched")}
        }
        Ok(())
    }
} 

pub trait OperatorT {
    fn execute(&self) -> Result<(), ExecError> {
        Ok(())
    }
}


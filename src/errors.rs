use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub struct ExecError {
    pub(crate) message: String
}

impl Error for ExecError {}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}


impl From<std::io::Error> for ExecError {
    fn from(error: std::io::Error) -> Self {
        ExecError {
            message: format!("Io error on exec: {}", error),
        }
    }
}

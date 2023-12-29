use std::process::Command;
use crate::operators::OperatorT;
use crate::errors::ExecError;

#[derive(PartialEq, Debug, Eq)]
pub struct ShellOperator {
    pub(crate) shell: String,
    pub(crate) command: String
}

impl OperatorT for ShellOperator {
    fn execute(&self) -> Result<(), ExecError> {

        let output = Command::new(&self.shell)
            .arg("/c")
            .arg(&self.command)
            .output()?;

        println!("{}", String::from_utf8(output.stdout).unwrap());
        
        Ok(())
    }   
}

impl ShellOperator {
    pub fn new(shell: &str, command: &str) -> Self {
        ShellOperator { 
            shell: shell.to_string(),
            command: command.to_string()
        }
    }

   
}


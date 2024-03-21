use crate::task::{ScheduleType, Task, TaskId};
use crate::Result;

pub const HELP: &'static str = "
usage: 
    add {task}    add and schedule new task
    list          list tasks
    drain         stop scheduling of task
    kill          kill and remove task from schedule
    EXIT          exit and close client";

pub enum ClientCommand {
    Add(Task),
    Help,
    List,
    Drain(TaskId),
    Kill(TaskId),
    Noop,
    Error(String),
    Exit,
}

impl ClientCommand {
    pub fn from_str(data: &str) -> Result<Self> {
        let mut parts = data.split(" ");

        println!("{:?}", data);

        // Match main command
        match parts.next() {
            Some(cmd) => match cmd {
                "ADD" => {
                    let task_id = match parts.next() {
                        Some(id) => id,
                        None => {
                            return Err("no task_id provided".into());
                        }
                    };

                    let schedule = match parts.next() {
                        Some(part) => ScheduleType::from_str(part)?,
                        None => {
                            return Err("no schedule provided".into());
                        }
                    };

                    let cmd: Vec<&str> = parts.collect();

                    // .ok_or("no cmd provided")?;

                    // let retries = parts.next().unwrap_or("1").parse()?;

                    let task = Task::new(task_id, schedule, cmd.join(" ").as_str(), 0);

                    return Ok::<ClientCommand, _>(ClientCommand::Add(task));
                }
                "LIST" => return Ok(ClientCommand::List),
                "DRAIN" => {
                    let task_id = match parts.next() {
                        Some(tid) => tid,
                        None => return Err("no task id provided".into()),
                    };

                    return Ok(ClientCommand::Drain(task_id.to_string()));
                }
                "KILL" => {
                    let task_id = match parts.next() {
                        Some(tid) => tid,
                        None => return Err("no task id provided".into()),
                    };

                    return Ok(ClientCommand::Kill(task_id.to_string()));
                }
                "HELP" => {
                    return Ok(ClientCommand::Help);
                }
                "EXIT" => {
                    return Ok(ClientCommand::Exit);
                }
                _ => Err(format!("Invalid Command {}", cmd)),
            },
            None => Err("No cmd provided".into()),
        }?;

        Err("somethin".into())
    }
}

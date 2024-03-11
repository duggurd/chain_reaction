use tokio::time::Duration;
use tokio::process::Command;
use std::time::SystemTime;

use tracing::{event, Level};

use crate::Result;


pub type TaskId = String;

#[derive(Debug, Clone)]
pub enum ScheduleType {
    Interval(Duration),
    DownStream(TaskId),
    Once
}

#[derive(Debug, Clone)]
pub struct Task {
    pub schedule: ScheduleType,
    pub cmd: String,
    pub retries: u16,
    pub task_id: TaskId
}


#[derive(Debug, Clone)]
pub struct TaskInstance {
    pub instance_id: String,
    pub task: Task,
    // pub start_time: DateTime<Local>,
    pub exec_at: SystemTime,
    pub logs: String,
    pub retry_num: u16,
    pub(crate) kill: bool
}

impl Task {
    pub fn new(
        task_id: &str,
        schedule: ScheduleType, 
        cmd: &str, 
        retries: Option<u16>
    ) -> Self {

        // Validate configs

        // Check task_id is unique

        // Check task_type Binary exists
        
        // Return error immedately before creating anything if any check fails

        Task { 
            task_id: task_id.to_string(), 
            schedule: schedule, 
            cmd: cmd.to_string(), 
            retries: retries.unwrap_or_default(), 
        }
    }
}

impl TaskInstance {
    pub fn new(task: Task, exec_at: std::time::SystemTime, retry_num: u16) -> Self {
        
        TaskInstance { 
            instance_id: format!(
                "{}_{}", 
                task.task_id, 
                std::time::SystemTime::now()
                    .duration_since(
                        std::time::SystemTime::UNIX_EPOCH
                    ).unwrap().as_secs()
                ), 
            task: task, 
            exec_at: exec_at, 
            logs: String::new(), 
            retry_num:  retry_num,
            kill: false
        }
    }

    pub async fn exec(&self) -> bool {
        event!(Level::TRACE, id=self.instance_id, cmd=self.task.cmd, "exec");
        
        let cmd = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("/C")
                .arg(&self.task.cmd)
                .spawn()
                .unwrap()

        } else {
            Command::new("sh")
                .arg("-c")
                .arg(&self.task.cmd)
                .arg("2>&1")
                .spawn()
                .unwrap()
        };

        let status = match cmd.wait_with_output().await {
            Ok(s) => s,
            Err(_e) => return false,
        };
      
        match status.status.success() {
            true => {
                event!(Level::TRACE, id=self.instance_id, "success");  
                return true
            },
            false => {

                let err = String::from_utf8(status.stdout)
                    .expect("failed to unpack vec utf-8");

                event!(Level::WARN, id=self.instance_id, err=err, "failed");
                return false
            }
        }

    }
}


impl PartialEq for TaskInstance {
    fn eq(&self, other: &Self) -> bool {
        self.instance_id == other.instance_id
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for TaskInstance { }


impl Ord for TaskInstance {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (&self.exec_at, &other.exec_at) {
            (time1, time2) => {
                time2.cmp(time1) // Reversing ordering of BHeap
            }
            
            _ => std::cmp::Ordering::Equal,
        }
    }
}

impl PartialOrd for TaskInstance {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match(&self.exec_at, &other.exec_at) {
            (time1, time2) => {
                Some(time2.cmp(time1)) // Reversing ordering of BHeap
            }
            _ => None
        }
    }
}
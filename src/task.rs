use std::process::ExitStatus;

use crate::schedule::Schedule;
use crate::unique_id::generate_unique_id;

use tokio::time::Duration;
use tokio::time::Instant;
use tokio::process::{Command};


#[derive(Debug)]
pub struct Task {
    // pub task_id: String,
    // pub(crate) schedule: Schedule,
    pub interval: Duration,
    unique_id: u64,
    // pub start_time: DateTime<Local>,
    pub next_exec: Instant,
    cmd: String
}

impl Task {
    pub fn new(secs: u64, cmd: String) -> Self {
        let interval = Duration::from_secs(secs);

        Task {
            interval: interval,
            cmd: cmd,
            next_exec: Instant::now() + interval,
            unique_id: generate_unique_id()
        }
    }

    pub async fn exec(&self) -> Result<String, String>{
        
        let cmd = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("/C")
                .arg(&self.cmd)
                .spawn()
                .unwrap()

        } else {
            Command::new("sh")
                .arg("-c")
                .arg(&self.cmd)
                .spawn()
                .unwrap()
        };

        match cmd.wait_with_output().await {
            Ok(status) => {
                if status.status.success() {
                    return Ok(String::from_utf8(status.stdout).expect("failed to unpack vec utf-8"))
                } else {
                    return Err(String::from_utf8(status.stderr).expect("failed to unpack vec utf-8"))
                }
            } ,
            Err(e) => Err(e.to_string()),
        }

    }
}


// impl Task {
//     pub fn new(
//         task_id: &str, 
//         start_time: DateTime<Local>,
//         duration: Option<Duration>,
//         cmd: &str,
//         skip_first_run: Option<bool>
//     ) -> Self {
        
//         let schedule = Schedule::new(start_time, duration, skip_first_run);

//         let cmd = if cfg!(target_os = "windows") {
//             Command::new("cmd")
//                     .args(["/C", "echo hello"])
//         } else {
//             Command::new("sh")
//                     .arg("-c")
//                     .arg("echo hello")
//         };

//         Task { 
//             task_id: task_id.to_string(), 
//             unique_id: generate_unique_id(),
//             schedule: schedule, 
//             next_exec: start_time, 
//             cmd: *cmd
//         }
//     }

//     // fn from_yaml(yaml: &str) -> Self {
//     //     let parsed_yaml = Yaml::from_str(yaml);
//     // }

//     pub(crate) fn calc_next_time(mut self) -> Option<Task>{
//         match &self.schedule.duration {
//             Some(dur) => {
                
//                 self.next_exec += *dur;
//                 Some(self)
//             }
//             _ => {None}
//         }
//     }

//     pub(crate) async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>>{
//         log::debug!("Executing task: {}", self.task_id);

//         self.cmd.spawn()?.wait().await?;
//         Ok(())
//     }

// }

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id == other.unique_id
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for Task { }


impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (&self.next_exec, &other.next_exec) {
            (time1, time2) => {
                time2.cmp(time1) // Reversing ordering of BHeap
            }
            
            _ => std::cmp::Ordering::Equal,
        }
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match(&self.next_exec, &other.next_exec) {
            (time1, time2) => {
                Some(time2.cmp(time1)) // Reversing ordering of BHeap
            }
            _ => None
        }
    }
}
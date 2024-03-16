use std::time::SystemTime;
use tokio::process::Command;
use tokio::time::Duration;

use tracing::{event, Level};

use crate::Result;

pub type TaskId = String;

/// Describes how a task is scheduled:
/// - `Interval`: task is executed every `Duration`
/// - `DownStream` [`TaskId`]: task has a down-stream dependency, and that task is executed on completion
/// - `Once`: task is scheduled and executed once (with retries)
#[derive(Debug, Clone)]
pub enum ScheduleType {
    Interval(Duration),
    DownStream(TaskId),
    Once,
}

/// Task configuration
/// Describes how to schedule and execute a task
#[derive(Debug, Clone)]
pub struct Task {
    pub schedule: ScheduleType,
    pub cmd: String,
    pub retries: u16,
    pub task_id: TaskId,
}

/// Actual scheduled instance of a task
#[derive(Debug, Clone)]
pub struct TaskInstance {
    pub instance_id: String,
    pub task: Task,
    pub exec_at: SystemTime,
    pub logs: String,
    pub retry_num: u16,
    pub(crate) kill: bool,
}

impl Task {
    pub fn new(task_id: &str, schedule: ScheduleType, cmd: &str, retries: u16) -> Self {
        // Validate configs

        // Check task_id is unique

        // Check task_type Binary exists

        // Return error immedately before creating anything if any check fails

        Task {
            task_id: task_id.to_string(),
            schedule: schedule,
            cmd: cmd.to_string(),
            retries: retries,
        }
    }
}

impl TaskInstance {
    /// Create a new [`TaskInstance`] instance
    pub fn new(task: Task, exec_at: SystemTime, retry_num: u16) -> Self {
        TaskInstance {
            instance_id: format!(
                "{}_{}",
                task.task_id,
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ),
            task: task,
            exec_at: exec_at,
            logs: String::new(),
            retry_num: retry_num,
            kill: false,
        }
    }

    /// Execute the task as described in task ([`Task`]).
    pub async fn exec(&self) -> bool {
        event!(
            Level::TRACE,
            id = self.instance_id,
            cmd = self.task.cmd,
            "exec"
        );

        let cmd = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("/C")
                .arg(&self.task.cmd)
                .output()
                .await
                .unwrap()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(&self.task.cmd)
                .arg("2>&1")
                .output()
                .await
                .unwrap()
        };

        // cmd.

        // let status = match cmd.wait_with_output().await {
        //     Ok(s) => s,
        //     Err(_e) => return false,
        // };

        match cmd.status.success() {
            true => {
                event!(Level::TRACE, id = self.instance_id, "success");
                return true;
            }
            false => {
                let err = String::from_utf8(cmd.stdout).expect("failed to unpack vec utf-8");

                event!(Level::WARN, id = self.instance_id, err = err, "failed");
                return false;
            }
        }
    }
}

impl ScheduleType {
    pub(crate) fn from_str(data: &str) -> Result<Self> {
        match data.trim() {
            "once" => Ok(Self::Once),

            s => {
                let mut parts = s.split(":");

                match parts.next() {
                    Some(st) => match st {
                        "dstream" => {
                            if let Some(s) = parts.next() {
                                return Ok(Self::DownStream(s.to_string()));
                            } else {
                                return Err("no task_id provided".into());
                            };
                        }
                        "interval" => {
                            if let Some(d) = parts.next() {
                                let value: u64 = match d[0..d.len() - 1].parse() {
                                    Ok(n) => n,
                                    Err(e) => return Err(e.into()),
                                };

                                let duration = match d.chars().last().unwrap() {
                                    'n' => Duration::from_nanos(value),
                                    's' => Duration::from_secs(value),
                                    'm' => Duration::from_secs(value * 60),
                                    'h' => Duration::from_secs(value * 60 * 60),
                                    _ => return Err("invalid time specifier".into()),
                                };

                                return Ok(Self::Interval(duration));
                            } else {
                                return Err("invalid ScheduleType provided".into());
                            }
                        }
                        _ => Err("incorrect scheduletype".into()),
                    },

                    None => return Err("invalid syntax for scheduletype".into()),
                }
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

impl Eq for TaskInstance {}

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
        match (&self.exec_at, &other.exec_at) {
            (time1, time2) => {
                Some(time2.cmp(time1)) // Reversing ordering of BHeap
            }
            _ => None,
        }
    }
}

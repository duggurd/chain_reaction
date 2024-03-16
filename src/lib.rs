// use std::time::SystemTime;
// use tokio::io::Error;

// use tracing::Level;

mod errors;
pub mod scheduler;
pub mod task;
// pub use task::{Task, TaskInstance, ScheduleType, TaskId};
// use scheduler::Scheduler;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

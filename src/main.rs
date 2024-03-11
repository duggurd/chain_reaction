use tokio::time::Duration;
use std::{collections::BinaryHeap, time::SystemTime};
use std::error::Error;
use std::collections::HashMap;

use tracing::{event, Level, span};


mod task;
mod scheduler;
use task::{Task, TaskInstance, ScheduleType, TaskId};
use scheduler::Scheduler;


type Result<T> = std::result::Result<T, Box<dyn Error>>;


#[tokio::main]
async fn main() -> Result<()> { 

    let subscriber = tracing_subscriber::fmt()
            .compact()
            .with_max_level(Level::TRACE)
            .finish();

    tracing::subscriber::set_global_default(subscriber)?;
  

    let echo_task = Task::new(
        "echo_task",
        ScheduleType::Interval(std::time::Duration::from_secs(5)), 
        "echo hello!", 
        Some(0)
    );

    let mut scheduler = Scheduler::new();

    scheduler.add_task(echo_task, SystemTime::now()).await?;

    scheduler.run().await?;
      
    Ok(())
}

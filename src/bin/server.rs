use std::time::SystemTime;

use tracing::Level;

use chainz::Result;
use chainz::scheduler::Scheduler;
use chainz::task::{Task, ScheduleType};


#[tokio::main]
async fn main() -> Result<()> { 

    let subscriber = tracing_subscriber::fmt()
            .compact()
            .with_max_level(Level::INFO)
            .finish();

    tracing::subscriber::set_global_default(subscriber)?;
  

    let echo_task = Task::new(
        "echo_task",
        ScheduleType::Interval(std::time::Duration::from_secs(1)), 
        "echo hello", 
        0
    );

    let mut scheduler = Scheduler::new();

    scheduler.add_task(echo_task, SystemTime::now()).await?;

    scheduler.run().await?;
      
    Ok(())
}

use tracing::Level;

use chainz::server::Server;
use chainz::task::{ScheduleType, Task};
use chainz::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // let echo_task = Task::new(
    //     "echo_task",
    //     ScheduleType::Interval(std::time::Duration::from_secs(1)),
    //     "echo hello",
    //     0,
    // );

    let mut server = Server::new("0.0.0.0:3333").await;

    server.run().await?;

    // let mut scheduler = Scheduler::new();

    // scheduler.add_task(echo_task, SystemTime::now())?;

    // scheduler.run().await?;

    Ok(())
}

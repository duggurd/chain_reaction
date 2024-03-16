use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::time::Duration;
use tracing::{event, span, Level};

use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::task::{ScheduleType, Task, TaskId, TaskInstance};
use crate::Result;

/// Schedules and manages lifecycle of [`Task`]s
pub struct Scheduler {
    pub tasks: HashMap<TaskId, Task>,
    pub task_q: BinaryHeap<TaskInstance>,
    drain: Vec<TaskId>,
}

pub enum ClientCommand {
    Add(Task),
    List,
    Drain(TaskId),
    Kill(TaskId),
    Noop,
}

impl ClientCommand {
    fn from_string(data: String) -> Result<Self> {
        let mut parts = data.split(" ");

        // Match initial command
        match parts.next() {
            Some(cmd) => match cmd.trim().to_uppercase().as_str() {
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

                    let cmd = parts.next().ok_or("no cmd provided")?;

                    let retries = parts.next().unwrap_or("0").parse()?;

                    let task = Task::new(task_id, schedule, cmd, retries);

                    return Ok::<ClientCommand, _>(ClientCommand::Add(task));
                }
                "LIST" => {
                    todo!()
                }
                "DRAIN" => {
                    todo!()
                }
                "KILL" => {
                    todo!()
                }
                _ => Err::<&str, &str>("Invalid Command".into()),
            },
            None => Err("No cmd provided".into()),
        }?;

        Err("something".into())
    }
}

struct Server {
    listener: TcpListener,
    scheduler: Arc<Mutex<Scheduler>>,
}

impl Server {
    async fn new<A>(address: A) -> Self
    where
        A: ToSocketAddrs,
    {
        Self {
            listener: TcpListener::bind(address).await.unwrap(),
            scheduler: Arc::new(Mutex::new(Scheduler::new())),
        }
    }

    async fn handle_request(&self, mut stream: TcpStream) {
        let sched = self.scheduler.clone();
        tokio::spawn(async move {
            let mut buf = String::new();
            stream.read_to_string(&mut buf).await.unwrap();

            let command = match ClientCommand::from_string(buf) {
                Ok(cmd) => cmd,
                Err(e) => {
                    // stream.write(e.to_string().as_bytes()).await;
                    ClientCommand::Noop
                }
            };

            match command {
                ClientCommand::Add(task) => {
                    match sched.lock().await.add_task(task, SystemTime::now()).await {
                        Ok(()) => {
                            // stream.write(b"task successfully added").await;
                        }
                        Err(e) => {
                            // stream.write(format!("failed to add task {}", e).as_bytes()).await;
                        }
                    }
                }

                ClientCommand::Drain(_task_id) => {
                    todo!();
                }
                ClientCommand::Kill(_task_id) => {
                    todo!();
                }
                ClientCommand::List => {
                    stream
                        .write(format!("{:?}", sched.lock().await.tasks).as_bytes())
                        .await
                        .unwrap();
                }

                ClientCommand::Noop => {}
            };
        });
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((stream, _addr)) => {
                    self.handle_request(stream).await;
                }
                Err(e) => {}
            }
        }
    }
}

impl Scheduler {
    /// Create new scheduler instance
    pub fn new() -> Self {
        Scheduler {
            tasks: HashMap::new(),
            task_q: BinaryHeap::<TaskInstance>::new(),
            drain: Vec::new(),
        }
    }

    /// Add new task and schedule task
    pub async fn add_task(&mut self, task: Task, start_time: SystemTime) -> Result<()> {
        event!(Level::INFO, id = task.task_id, "add");

        match self.tasks.contains_key(&task.task_id) {
            true => {
                event!(Level::ERROR, id = task.task_id, "already_exists");
                return Err(format!("task '{}' already exists", task.task_id).into());
            }
            false => {
                event!(Level::INFO, id = task.task_id, "inserted");

                let task2 = task.clone();
                let task3 = task.clone();
                self.tasks.insert(task.task_id, task2);

                self.schedule_task(task3, start_time, 0).await?;
            }
        }

        Ok(())
    }

    /// Actually schedule task
    /// Creates a new [`TaskInstance`] and adds it to the queue
    pub async fn schedule_task(
        &mut self,
        task: Task,
        exec_at: SystemTime,
        retry_num: u16,
    ) -> Result<()> {
        let task2 = task.clone();
        let task3 = task.clone();

        let ti = TaskInstance::new(task2, exec_at, retry_num);

        let inst_id = ti.instance_id.clone();

        self.task_q.push(ti);

        event!(
            Level::TRACE,
            id = task3.task_id,
            inst_id = inst_id,
            "scheduled"
        );

        Ok(())
    }

    /// Exec tasks at top of schedule
    /// Reschedule if config says so
    /// Trigger down-stream tasks
    /// Sleep until trigger of next task or until new task is added
    async fn poll(&mut self) -> Result<()> {
        let span = span!(Level::TRACE, "poll");
        let _enter = span.enter();

        tracing::trace!("polling");

        while self
            .task_q
            .peek()
            .is_some_and(|ti| ti.exec_at < SystemTime::now())
        {
            // Should never return None
            let next_task = self.task_q.pop().unwrap();
            event!(
                Level::INFO,
                id = next_task.task.task_id,
                inst_id = next_task.instance_id,
                "exec"
            );

            match next_task.exec().await {
                // reschedule if failed and less than retry, with backoff
                false => {
                    if next_task.retry_num >= next_task.task.retries {
                        return Err("Task failed, no more retries".into());
                    } else {
                        self.schedule_task(
                            next_task.task,
                            SystemTime::now()
                                + Duration::from_secs(2 * (next_task.retry_num + 1) as u64),
                            next_task.retry_num + 1,
                        )
                        .await?;
                    }
                }
                true => {
                    let mut drain = false;

                    for (i, d) in self.drain.iter_mut().enumerate() {
                        if d == &next_task.task.task_id {
                            self.tasks.remove(d);
                            self.drain.swap_remove(i);
                            drain = false;
                            break;
                        }
                    }

                    if drain == false {
                        match next_task.task.schedule {
                            ScheduleType::DownStream(task_id) => {
                                self.schedule_task(
                                    self.tasks.get(&task_id).unwrap().to_owned(),
                                    SystemTime::now(),
                                    0,
                                )
                                .await?;
                            }
                            ScheduleType::Interval(duration) => {
                                self.schedule_task(
                                    next_task.task.to_owned(),
                                    SystemTime::now() + duration,
                                    0,
                                )
                                .await?;
                            }

                            // Ran successfully, no reschedule
                            ScheduleType::Once => (),
                        }
                    }
                }
            }
        }

        let sleep_dur = match self.task_q.peek() {
            Some(t) => t.exec_at.duration_since(SystemTime::now()).unwrap(),
            None => tokio::time::Duration::from_secs(2),
        };

        tokio::time::sleep(sleep_dur).await;

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        tracing::trace!("starting scheduler");

        loop {
            self.poll().await?;
        }
    }

    /// Drain task from schedule
    /// Scheduled tasks continue to run
    /// If task fails runs until no more retries left
    pub fn drain_task(&mut self, task_id: TaskId) -> Result<()> {
        self.drain.push(task_id);
        Ok(())
    }

    /// Immediately remove task from que and task map
    /// This is an expensive operation
    fn kill_task(&mut self, task_id: TaskId) -> Result<()> {
        todo!();

        // if !self.tasks.contains_key(&task_id) {
        //     return Err("Task does not exist".into())
        // } else {

        // }

        // Ok(())
    }
}

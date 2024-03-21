use std::collections::{BinaryHeap, HashMap};

use crate::task::{ScheduleType, Task, TaskId, TaskInstance};
use crate::Result;
use std::sync::Mutex;
use std::time::SystemTime;
use tokio::time::Duration;
use tracing::{event, span, Level};

/// Schedules and manages lifecycle of [`Task`]s
pub struct Scheduler {
    pub tasks: Mutex<HashMap<TaskId, Task>>,
    pub task_q: Mutex<BinaryHeap<TaskInstance>>,
    drain: Mutex<Vec<TaskId>>,
}

impl Scheduler {
    /// Create new scheduler instance
    pub fn new() -> Self {
        Scheduler {
            tasks: Mutex::new(HashMap::new()),
            task_q: Mutex::new(BinaryHeap::<TaskInstance>::new()),
            drain: Mutex::new(Vec::new()),
        }
    }

    /// Add new task and schedule task
    pub fn add_task(&self, task: Task, start_time: SystemTime) -> Result<()> {
        event!(Level::INFO, id = task.task_id, "add");

        let do_contain = self.tasks.lock().unwrap().contains_key(&task.task_id);

        match do_contain {
            true => {
                event!(Level::ERROR, id = task.task_id, "already_exists");
                return Err(format!("task '{}' already exists", task.task_id).into());
            }
            false => {
                event!(Level::INFO, id = task.task_id, "inserted");

                let task2 = task.clone();
                let task3 = task.clone();
                self.tasks.lock().unwrap().insert(task.task_id, task2);

                self.schedule_task(task3, start_time, 0)?;
            }
        }

        Ok(())
    }

    /// Actually schedule task
    /// Creates a new [`TaskInstance`] and adds it to the queue
    pub fn schedule_task(&self, task: Task, exec_at: SystemTime, retry_num: u16) -> Result<()> {
        event!(Level::TRACE, "scheduling task");
        let task2 = task.clone();
        let task3 = task.clone();

        let ti = TaskInstance::new(task2, exec_at, retry_num);

        let inst_id = ti.instance_id.clone();

        self.task_q.lock().unwrap().push(ti);

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
    async fn poll(&self) -> Result<()> {
        let span = span!(Level::TRACE, "poll");
        let _enter = span.enter();

        event!(Level::TRACE, "polling");

        while self
            .task_q
            .lock()
            .unwrap()
            .peek()
            .is_some_and(|ti| ti.exec_at < SystemTime::now())
        {
            // Should never return None
            let next_task = self.task_q.lock().unwrap().pop().unwrap();

            event!(
                Level::INFO,
                id = next_task.task.task_id,
                inst_id = next_task.instance_id,
                "exec"
            );

            match next_task.exec().await {
                // reschedule if failed and less than retry, with backoff
                Err(e) => {
                    event!(
                        Level::TRACE,
                        id = next_task.task.task_id,
                        err = e,
                        "task failed"
                    );

                    if next_task.retry_num >= next_task.task.retries {
                        event!(
                            Level::ERROR,
                            id = next_task.task.task_id,
                            "task failed, no more retries"
                        );
                    } else {
                        self.schedule_task(
                            next_task.task,
                            SystemTime::now()
                                + Duration::from_secs(2 * (next_task.retry_num + 1) as u64),
                            next_task.retry_num + 1,
                        )?;
                    }
                }
                Ok(()) => {
                    let mut drain = false;

                    for (i, d) in self.drain.lock().unwrap().iter_mut().enumerate() {
                        if d == &next_task.task.task_id {
                            self.tasks.lock().unwrap().remove(d);
                            self.drain.lock().unwrap().swap_remove(i);
                            drain = false;
                            break;
                        }
                    }

                    if drain == false {
                        match next_task.task.schedule {
                            ScheduleType::DownStream(task_id) => {
                                let task =
                                    self.tasks.lock().unwrap().get(&task_id).unwrap().clone();

                                self.schedule_task(task, SystemTime::now(), 0)?;
                            }

                            ScheduleType::Interval(duration) => {
                                self.schedule_task(
                                    next_task.task.to_owned(),
                                    SystemTime::now() + duration,
                                    0,
                                )?;
                            }

                            // Ran successfully, no reschedule
                            ScheduleType::Once => {
                                self.tasks.lock().unwrap().remove(&next_task.task.task_id);
                            }
                        }
                    }
                }
            }
        }

        let sleep_dur = match self.task_q.lock().unwrap().peek() {
            Some(t) => t.exec_at.duration_since(SystemTime::now()).unwrap(),
            None => tokio::time::Duration::from_secs(2),
        };

        tokio::time::sleep(sleep_dur).await;

        Ok(())
    }

    pub async fn run(&self) {
        event!(Level::TRACE, "starting scheduler");

        loop {
            self.poll().await.unwrap();
        }
    }

    /// Drain task from schedule
    /// Scheduled tasks continue to run
    /// If task fails runs until no more retries left
    pub fn drain_task(&mut self, task_id: TaskId) -> Result<()> {
        self.drain.lock().unwrap().push(task_id);
        Ok(())
    }

    /// Immediately remove task from que and task map
    /// This is an expensive operation
    #[allow(dead_code)]
    fn kill_task(&mut self, _task_id: TaskId) -> Result<()> {
        todo!();

        // if !self.tasks.contains_key(&task_id) {
        //     return Err("Task does not exist".into())
        // } else {

        // }

        // Ok(())
    }
}

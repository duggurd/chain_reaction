use std::borrow::BorrowMut;
use std::collections::{BinaryHeap, HashMap};
use tokio::time::Duration;
use tracing::{event, Level, span};
use std::time::SystemTime;

use crate::Result;
use crate::{Task, TaskInstance, TaskId, ScheduleType};


pub struct Scheduler {
    pub tasks: HashMap<TaskId, Task>,
    pub task_q: BinaryHeap<TaskInstance>,
    drain: Vec<TaskId>
}


impl Scheduler {

    pub fn new() -> Self {
        Scheduler { 
            tasks: HashMap::new(),
            task_q: BinaryHeap::<TaskInstance>::new(),
            drain: Vec::new()
        }
    }

    /// Add new task and schedule task
    pub async fn add_task(
        &mut self, 
        task: Task, 
        start_time: SystemTime
    ) -> Result<()> {

        event!(Level::INFO, id=task.task_id, "add");
        
        match self.tasks.contains_key(&task.task_id) {
            true => {
                event!(Level::ERROR, id=task.task_id, "already_exists");
                return Err(format!("task '{}' already exists", task.task_id).into())
            },
            false => {
                event!(Level::INFO, id=task.task_id, "inserted");
                
                let task2 = task.clone();
                let task3 = task.clone();
                self.tasks.insert(task.task_id, task2);

                self.schedule_task(task3, start_time, 0).await?;
            }
        }
        
        Ok(())
    }

    pub async fn schedule_task(
        &mut self, 
        task: Task, 
        exec_at: SystemTime, 
        retry_num: u16
    ) -> Result<()> {
        
        let task2 = task.clone();
        let task3 = task.clone();
        
        let ti = TaskInstance::new(
            task2, 
            exec_at, 
            retry_num
        );

        let inst_id = ti.instance_id.clone();

        self.task_q.push(
          ti  
        );

        event!(Level::TRACE, id=task3.task_id, inst_id=inst_id, "scheduled");
        
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

        while self.task_q.peek().is_some_and(|ti| ti.exec_at < SystemTime::now()) {
            
            
            // Should never return None
            let next_task = self.task_q.pop().unwrap();
            event!(Level::INFO, id=next_task.task.task_id, inst_id=next_task.instance_id, "exec");
          
            match next_task.exec().await {
                // reschedule if failed and less than retry, with backoff
                false => {
                    if next_task.retry_num >= next_task.task.retries {
                        return Err("Task failed, no more retries".into());
                    } else {
                        self.schedule_task(
                            next_task.task, 
                            SystemTime::now() + Duration::from_secs(2*(next_task.retry_num+1) as u64), 
                            next_task.retry_num + 1
                        ).await?;
                    }
                },
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
                                    0
                                ).await?;
                            },
                            ScheduleType::Interval(duration) => {
                                self.schedule_task(
                                    next_task.task.to_owned(), 
                                    SystemTime::now() + duration, 
                                    0
                                ).await?;
                            },
            
                            // Ran successfully, no reschedule
                            ScheduleType::Once => ()
                        }
                    }
                }
            }
        }

        let sleep_dur = match self.task_q.peek() {
            Some(t)  => {
                t.exec_at.duration_since(SystemTime::now()).unwrap()
            },
            None => tokio::time::Duration::from_secs(2)
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
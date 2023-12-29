use std::collections::{BinaryHeap, HashMap};
use std::fs::{read_dir, read_to_string};
use std::io::Error;
use log::{self, LevelFilter, error};
use simple_logger::SimpleLogger;
use tokio::time::{interval, Duration};
use chrono::Local;

use std::sync::{Arc, Mutex};

use crate::task::Task;



impl Scheduler {
    pub async fn new(poll_sec: Option<u64>, log_level: LevelFilter, address: Option<&str>) -> Self {
        assert!(poll_sec.is_some_and(|p| p > 1));

        SimpleLogger::new()
            .with_level(log_level)
            .env()
            .init()
            .unwrap();

        let listener = TcpListener::bind(address.unwrap_or("127.0.0.0:9876")).await.expect("Failed to bind address");
        
        return Scheduler {
            task_q: BinaryHeap::<Task>::new(),
            poll_sec: poll_sec.unwrap_or(10),
            tcp_listener: listener
            // tasks_path: tasks_path.unwrap_or("/tasks".to_string())
        } 
    }

    pub fn add_task(&mut self, task: Task) {
        log::debug!("Adding task: {}, next scheduled run: {}", task.task_id, task.next_exec);

        self.task_q.push(task);
    }

    fn poll(&mut self) {
        
        if ! self.task_q.is_empty() {

            log::debug!("Polling, {} tasks in q", self.task_q.len());

            while self.task_q
                .peek()
                .is_some_and(|x| x.next_exec <= Local::now()) {
                
                // Probably better to get a mut ref, execute and if successfull (and within retries) remove task from q
                let next_task = self.task_q.pop().unwrap();
                
                log::info!("Executing: {}", next_task.task_id);
                
                //Check for error on exec
                match next_task.execute() {
                    Ok(()) => {}
                    Err(err) => error!("Task: {} failed, with err: {}", &next_task.task_id, err)
                }
                
                // Reschedule task
                if next_task.schedule.duration.is_some() {
                    log::debug!("Scheduling next run of task: {}", next_task.task_id);
                    
                    match next_task.calc_next_time() {
                        Some(task) => {
                            self.task_q.push(task);
                        },

                        _ => {}
                    }
                }    
            }
        } else {
            log::debug!("poll skipped; no tasks in q");
        }
       
    }

    pub fn run(&mut self) {
        log::info!("schedule started, {} tasks in q", {self.task_q.len()});
        
        // let interval = interval(Duration::from_secs(self.poll_sec));

        let task_q = Arc::new(HashMap::<Task> {
            map: Mutex::new(self.task_q),
        });
        
        loop {
            match self.tcp_listener.accept().await {
                Ok((socket, _)) => {
                    task_q = self.task_q.clone();
                }

            }
            // other things to do on each cycle, for example refresh tasks every x run
            
            // interval.tick().await?;
            self.poll();
            std::thread::sleep(std::time::Duration::from_secs(self.poll_sec));
        }
    }

    // Check if any files have changed since last read time
    // If so update
    // fn read_tasks(self) -> Result<(), Error>{
    //     let mut files =  read_dir(self.tasks_path)?;
    //     let mut yaml_tasks = Vec::<String>::new();
        
    //     for fp in files {
    //         yaml_tasks.push(
    //             read_to_string(fp?.path())?
    //         );
    //     }

    //     Ok(())
        
    // }
}
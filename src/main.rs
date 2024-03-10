use tokio::time::Duration;

mod task;

use std::sync::Mutex;
use std::collections::BinaryHeap;
use std::error::Error;

use task::Task;

struct Scheduler {
    task_q: Mutex<BinaryHeap<Task>>
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{    
    let initial_task_q = BinaryHeap::<Task>::new();

    let mut scheduler = Scheduler {
        task_q: Mutex::new(initial_task_q)
    };

    scheduler.run().await;
      
    Ok(())
}


impl Scheduler {
    async fn poll(&mut self) {
        let mut guard = self.task_q.lock().unwrap();

        match guard.pop() {
            Some(t) => match t.exec().await {
                Ok(a) => todo!(),
                Err(e) => todo!(),
            },
            _ => {}
        };

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    async fn run(&mut self) {
        loop {
            self.poll();
        }
    }
}

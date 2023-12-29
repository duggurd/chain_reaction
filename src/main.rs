mod task;
mod unique_id;
mod schedule;

use tokio::net::TcpListener;
use tokio::time::{Duration, Instant};
use tokio_stream::StreamExt;
use futures::SinkExt;
use tokio_util::codec::{Framed, LinesCodec};

use simple_logger::SimpleLogger;
use log;

use std::borrow::Borrow;
use std::clone;
use std::sync::{Arc, Mutex};
use std::collections::BinaryHeap;
use std::error::Error;

use task::Task;


enum Request {
    List,
    Create { secs: u64, cmd: String }
}

enum Response {
    List {
        scheduler: String
    },

    Create {
        secs: u64, cmd: String
    },
    
    Error { 
        msg: String 
    }
}

struct Scheduler {
    task_q: Mutex<BinaryHeap<Task>>
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let _ = SimpleLogger::new().with_level(log::LevelFilter::Debug).init();
    
    let initial_task_q = BinaryHeap::<Task>::new();

    let scheduler = Arc::new(Scheduler {
        task_q: Mutex::new(initial_task_q)
    });

    scheduler.task_q.;
    log::debug!("starting scheduler");

    tokio::join!(
        scheduler.,
        tokio::spawn(get_updates(scheduler.clone())),
    );

    loop {

        let scheduler = scheduler.clone();

        tokio::spawn(async move { &scheduler.clone().poll() });

        match listener.accept().await {
            Ok((socket, _)) => {
                log::debug!("got connection");
                
                let scheduler = Arc::clone(&scheduler);

                tokio::spawn(async move {
                    let mut lines = Framed::new(socket, LinesCodec::new());
                    
                    while let Some(result) = lines.next().await {
                        match result {
                            Ok(line) => {
                                log::debug!("handling request!");
                                let response = handle_request(&line, &scheduler.clone());

                                let response = response.serialize();
                                
                                if let Err(e) = lines.send(response).await {
                                    println!("error on sending response; error = {:?}", e);
                                }
                            }
                            Err(e) => {
                                println!("error on decoding from socket; error = {:?}", e);
                            }
                        }
                    }
                });
            }
            Err(e) => {
                println!("Failed to accept socket; error {:?}", e);
            }
        }
    }
    Ok(())
}


fn handle_request(line: &str, scheduler: &Arc<Scheduler>) -> Response {
    let request = match Request::parse(line) {
        Ok(req) => req,
        Err(e) => return Response::Error { msg: e },
    };

    log::debug!("locking task q");
    let mut scheduler = scheduler.task_q.lock().unwrap();
    
    log::debug!("locked task q");
    match request {
        Request::List => Response::List { scheduler: format!("{:?}", scheduler)},
        Request::Create { secs, cmd } => {
            log::debug!("creating task");
            scheduler.push(Task::new(secs, cmd.clone()));
            Response::Create { secs: secs, cmd: cmd }
        }
    }

}

impl Request {
    fn parse(line: &str) -> Result<Request, String>{
        let mut parts = line.splitn(3, " ");
        match parts.next() {
            Some("LIST") => {
                if parts.next().is_some() {
                    return Err("Nithing should follow after LIST".into());
                }
                Ok(Request::List)
            }
            Some("CREATE") => {
                let secs: u64 = parts.next().ok_or("Create needs Time")?
                    .parse()
                    .map_err(|e| format!("{}", e))?;

                let cmd = parts.next().ok_or("Create needs a CMD")?;
                if parts.next().is_some() {
                    return Err("CREATE should not be followed by anything after cmd".into());
                }
                Ok(Request::Create { 
                    secs: secs, 
                    cmd: cmd.to_string() 
                })
            }
            Some(cmd) => Err(format!("unknown command: {}", cmd)),
            None => Err("Empty input".into()),
        }
    }
}

impl Response {
    fn serialize(&self) -> String {
        match self {
            Response::List { scheduler } => format!("{:?}", scheduler),
            Response::Create { secs, cmd } => format!("Secs: {} Cmd: {}", secs, cmd ),
            Response::Error { msg } => format!("error: {}", msg),
        }
    }
}

impl Scheduler {
    async fn poll(&mut self) {
        let mut guard = self.task_q.lock().unwrap();

        match guard.pop() {
            Some(t) => match t.exec().await {
                Ok(a) => log::debug!("Succesfully exec task"),
                Err(e) => log::error!("failed to exec task: {}", e),
            },
            _ => {}
        }

        log::debug!("checking tasks");

        println!("{:?}", guard.peek());


        log::debug!("got to the end");
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

use std::ffi::CStr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tracing::{event, Level};

use tokio::net::{TcpStream, ToSocketAddrs};

use crate::command::{ClientCommand, HELP};
use crate::scheduler::Scheduler;
use crate::Result;

pub struct Server {
    listener: TcpListener,
    scheduler: Arc<Scheduler>,
}

impl Server {
    pub async fn new<A>(address: A) -> Self
    where
        A: ToSocketAddrs + std::fmt::Display + tracing::Value,
    {
        Self {
            listener: TcpListener::bind(address).await.unwrap(),
            scheduler: Arc::new(Scheduler::new()),
        }
    }

    async fn handle_request(&self, mut stream: TcpStream) {
        event!(Level::TRACE, "client connected");

        let sched = self.scheduler.clone();

        tokio::spawn(async move {
            // let (stream_reader, mut stream_writer) = stream.split();
            // let mut buf_stream_reader = BufReader::new(stream_reader);

            loop {
                let mut buf = [0; 1024];
                // buf.clear();
                let _bytes = stream.read(&mut buf).await.unwrap();
                // buf_stream_reader.read(buf )

                let c = CStr::from_bytes_until_nul(&buf).unwrap().to_str().unwrap();

                println!("{}", c == "TEST\r\n");

                event!(Level::INFO, c);

                let command = match ClientCommand::from_str(&c[0..c.len() - 2]) {
                    Ok(c) => c,
                    Err(e) => ClientCommand::Error(e.to_string()),
                };

                match command {
                    ClientCommand::Add(task) => {
                        let resp: String = match sched.add_task(task, SystemTime::now()) {
                            Ok(()) => "task successfully added".to_string(),
                            Err(e) => e.to_string(),
                        };

                        stream.write(resp.as_bytes()).await.unwrap();
                    }

                    ClientCommand::Drain(_task_id) => {
                        todo!();
                    }
                    ClientCommand::Kill(_task_id) => {
                        todo!();
                    }
                    ClientCommand::List => {
                        stream
                            .write(format!("{:?}", sched.tasks.lock().unwrap()).as_bytes())
                            .await
                            .unwrap();
                    }
                    ClientCommand::Help => {
                        stream.write(HELP.as_bytes()).await.unwrap();
                    }

                    ClientCommand::Noop => {
                        stream.write(b"nothing happened").await.unwrap();
                    }
                    ClientCommand::Error(e) => {
                        stream.write(e.to_string().as_bytes()).await.unwrap();
                    }
                    ClientCommand::Exit => {
                        event!(Level::TRACE, "closing connection to client");
                        stream.write(b"closing connection").await.unwrap();
                        stream.shutdown().await.unwrap();
                        stream.flush().await.unwrap();
                        return;
                    }
                };
                stream.flush().await.unwrap();
            }
        });
    }

    pub async fn run(&mut self) -> Result<()> {
        let addr = self.listener.local_addr()?.to_string(); // temp lazy solution, because im lazy

        event!(Level::INFO, address = addr, "starting server");

        let sched = self.scheduler.clone();

        tokio::join!(sched.run(), self.run_listener());

        Ok(())
    }

    async fn run_listener(&self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, _addr)) => {
                    self.handle_request(stream).await;
                }
                Err(e) => {
                    event!(
                        Level::ERROR,
                        err = format!("{}", e),
                        "failed to accept request"
                    );
                }
            }
        }
    }
}

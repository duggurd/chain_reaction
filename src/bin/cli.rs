use tokio::{
    io::{stdin, AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

pub struct Client {
    connection: TcpStream,
}

impl Client {
    async fn new<A: ToSocketAddrs>(address: A) -> Self {
        let connection = TcpStream::connect(address).await.unwrap();

        Client { connection }
    }

    async fn run(&mut self) {
        loop {
            let mut stdin = stdin();
            let mut buf: [u8; 1024] = [0; 1024];

            // let mut buf = String::new();
            let _ = stdin.read(&mut buf).await.unwrap();

            // println!("{}", String::from_utf8(buf.to_vec()).unwrap());

            self.connection.write(&buf).await.unwrap();
            self.connection.flush().await.unwrap();
            buf.fill(0);
            self.connection.read(&mut buf).await.unwrap();

            println!("{}", String::from_utf8(buf.to_vec()).unwrap());
        }
    }
}

#[tokio::main]
async fn main() {
    let mut client = Client::new("127.0.0.1:3333").await;

    client.run().await;
}

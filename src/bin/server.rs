use chainz::server::Server;
use chainz::Result;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let mut server = Server::new("0.0.0.0:3333").await;

    server.run().await?;

    Ok(())
}

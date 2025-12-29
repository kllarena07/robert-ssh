mod app;

use crate::server::AppServer;

mod server;

#[tokio::main]
async fn main() {
    let mut server = AppServer::new();
    server.run().await.expect("Failed running server");
}

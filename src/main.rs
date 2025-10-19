use tokio::sync::mpsc;

use mindustry_net::client::{Client, ClientEvent};

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = mpsc::channel(1024);

    let mut client = Client::new("127.0.0.1:6567".parse().unwrap(), "Swarm".to_string()).await;
    let state = client.state.clone();

    tokio::spawn(async move {
        loop {
            let event = receiver.recv().await.unwrap();
            match event {
                ClientEvent::MapLoaded => println!("> Map loaded!"),
                _ => {}
            }
        }
    });

    client.handle_packets(sender).await;
}

use mindustry_net::client::Client;
use mindustry_net::packet::Packet;

#[tokio::main]
async fn main() {
    // let mut client =
    // Client::new("130.162.212.232:6507".parse().unwrap(), "Swarm".to_string()).await;
    let mut client = Client::new("127.0.0.1:6567".parse().unwrap(), "Swarm".to_string()).await;
    let state = client.state.clone();

    loop {
        let packet = client.handle_packets().await.unwrap();
        match packet {
            Packet::SendMessageCall2 { message, .. } => println!("MGS: {message}"),
            Packet::WorldStream {
                wave,
                wave_time,
                tick,
                seed0,
                seed1,
                id,
                ..
            } => {
                println!("World loaded:");
                println!("Player Id: {id}");
                println!("Wave: {wave} {wave_time}");
                println!("Tick: {tick}");
                println!("Seed: {seed0} / {seed1}");
            }
            _ => {}
        }
    }
}

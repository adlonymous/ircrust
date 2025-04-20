mod client;

use client::Client;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{info, error};
use std::sync::Arc;
use tokio::sync::Mutex;

type Clients = Arc<Mutex<Vec<Arc<Client>>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("0.0.0.0:6667").await?;
    let clients = Arc::new(Mutex::new(Vec::new()));

    info!("IRC server running on port 6667...");

    loop {
        let(stream, addr) = listener.accept().await?;
        info!("New connection {:?}", addr);

        let clients = clients.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, clients).await {
                error!("Error handling client: {:?}", e);
            }
        });
    }
}

async fn handle_client(stream: TcpStream, clients: Clients) -> anyhow::Result<()> {
    let (reader, writer) = stream.into_split();

    let client = Arc::new(Client {
        nickname: Mutex::new(None),
        username: Mutex::new(None),
        writer: Mutex::new(writer),
    });

    {
        let mut clients_lock = clients.lock().await;
        clients_lock.push(client.clone());
    }

    let mut reader = BufReader::new(reader);
    let mut buffer = String::new();

    let welcome = b":rustirc 001 Welcome to Mous' IRC Server\r\n";
    {
        let mut w = client.writer.lock().await;
        w.write_all(welcome).await?;
    }

    while reader.read_line(&mut buffer).await? != 0 {
        let msg = buffer.trim();
        info!("Receved message: {:?}", msg);

        if msg.starts_with("NICK ") {
            let nick = msg[5..].trim().to_string();
            *client.nickname.lock().await = Some(nick);
        } else if msg.starts_with("USER "){
            let parts: Vec<&str> = msg.split_whitespace().collect();
            if parts.len() >= 2 {
                *client.username.lock().await = Some(parts[1].to_string());
            }
        } else if msg.starts_with("PRIVMSG ") {
            if let Some(idx) = msg.find(" :"){
                let message = &msg[idx + 2..];
                let sender = client.display_name().await;

                let clients_lock = clients.lock().await;
                for c in clients_lock.iter(){
                    let mut w = c.writer.lock().await;
                    let formatted = format!(":{} PRIVMSG {} :{}\r\n", sender, c.display_name().await, message);
                    if let Err(e) = w.write_all(formatted.as_bytes()).await {
                        error!("Error sending message to client: {:?}", e);
                    }
                }
            }
        }
        buffer.clear();
    }

    {
        let mut clients_lock = clients.lock().await;
        clients_lock.retain(|c| !Arc::ptr_eq(c, &client));
    }

    Ok(())
}

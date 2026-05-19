use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Sender, channel};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

// --- BAGIAN 1: HANDLE CONNECTION (TUTORIAL 2.3) ---
async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();

    // Kirim pesan sambutan
    let _ = ws_stream.send(Message::text("Welcome to chat!")).await;

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            // Tutorial 2.3: Tambahkan info IP dan Port pengirim
                            let formatted_msg = format!("{}: {}", addr, text);
                            let _ = bcast_tx.send(formatted_msg);
                        }
                    }
                    _ => break,
                }
            }
            msg = bcast_rx.recv() => {
                ws_stream.send(Message::text(msg?)).await?;
            }
        }
    }
    Ok(())
}

// --- BAGIAN 2: MAIN FUNCTION (TUTORIAL 2.2) ---
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Pakai port non-privileged yang tidak bentrok dengan service lokal umum.
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("Server listening on port 7878");

    let (bcast_tx, _) = channel(16);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);
        let bcast_tx = bcast_tx.clone();

        tokio::spawn(async move {
            // Membungkus TcpStream mentah menjadi WebSocketStream
            match ServerBuilder::new().accept(socket).await {
                Ok(ws_stream) => {
                    let _ = handle_connection(addr, ws_stream, bcast_tx).await;
                }
                Err(err) => eprintln!(
                    "Failed to accept WebSocket connection from {}: {}",
                    addr, err
                ),
            }
        });
    }
}

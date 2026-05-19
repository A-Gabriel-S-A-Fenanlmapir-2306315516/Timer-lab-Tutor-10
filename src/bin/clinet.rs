use futures_util::stream::StreamExt;
use futures_util::sink::SinkExt;
use http::Uri;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    // TUTORIAL 2.2: Koneksi ke port server lokal
    let (mut ws_stream, _) = ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:7878"))
        .connect()
        .await?;

    let mut stdin_lines = BufReader::new(tokio::io::stdin()).lines();

    loop {
        tokio::select! {
            line = stdin_lines.next_line() => {
                if let Ok(Some(text)) = line {
                    ws_stream.send(Message::text(text)).await?;
                } else { break; }
            }
            msg = ws_stream.next() => {
                if let Some(Ok(msg)) = msg {
                    if let Some(text) = msg.as_text() {
                        println!("{}", text);
                    }
                } else { break; }
            }
        }
    }
    Ok(())
}

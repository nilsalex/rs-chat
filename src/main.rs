use futures::{SinkExt, StreamExt};
use std::error::Error;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

async fn chat(
    stream1: WebSocketStream<TcpStream>,
    stream2: WebSocketStream<TcpStream>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr1 = stream1.get_ref().peer_addr()?;
    let addr2 = stream2.get_ref().peer_addr()?;

    let (mut sender1, mut receiver1) = stream1.split();
    let (mut sender2, mut receiver2) = stream2.split();

    sender1
        .send(Message::Text(format!(
            "Hi, you're chatting with {}.\n",
            addr2
        )))
        .await?;

    sender2
        .send(Message::Text(format!(
            "Hi, you're chatting with {}.\n",
            addr1
        )))
        .await?;

    loop {
        tokio::select! {
            item = receiver1.next() => {
                if let Some(msg_result) = item {
                    match msg_result {
                        Ok(msg) => sender2.send(msg).await?,
                        Err(err) => println!("{:?}", err)
                    }
                } else {
                    sender2.close().await?;
                    break
                }
            },
            item = receiver2.next() => {
                if let Some(msg_result) = item {
                    match msg_result {
                        Ok(msg) => sender1.send(msg).await?,
                        Err(err) => println!("{:?}", err)
                    }
                } else {
                    sender1.close().await?;
                    break
                }
            },
        }
    }

    println!("connection closed");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    let max_connections = 2;
    let semaphore = Arc::new(Semaphore::new(max_connections));

    loop {
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let (stream1, address1) = listener.accept().await?;
        let ws_stream1 = tokio_tungstenite::accept_async(stream1).await?;
        let (stream2, address2) = listener.accept().await?;
        let ws_stream2 = tokio_tungstenite::accept_async(stream2).await?;

        println!(
            "New connections from {} and {}. Starting chat!",
            &address1, &address2
        );

        tokio::spawn(async move {
            let _p = permit;
            chat(ws_stream1, ws_stream2).await
        });
    }
}

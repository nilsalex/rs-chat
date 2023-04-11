use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

async fn chat(mut stream1: TcpStream, mut stream2: TcpStream) -> std::io::Result<()> {
    let addr1 = stream1.peer_addr()?;
    let addr2 = stream2.peer_addr()?;

    stream1
        .write_all(format!("Hi, you're chatting with {}.\n", addr2).as_bytes())
        .await?;
    stream2
        .write_all(format!("Hi, you're chatting with {}.\n", addr1).as_bytes())
        .await?;

    let mut buffer1 = [0; 1024];
    let mut buffer2 = [0; 1024];

    loop {
        let future1 = stream1.read(&mut buffer1);
        let future2 = stream2.read(&mut buffer2);

        tokio::select! {
            n = future1 => {
                match n {
                    Ok(0) => {break;}
                    Ok(n) => {stream2.write_all(&buffer1[0..n]).await?;}
                    _     => {break;}
                }
            },
            n = future2 => {
                match n {
                    Ok(0) => {break;}
                    Ok(n) => {stream1.write_all(&buffer2[0..n]).await?;}
                    _     => {break;}
                }
            },
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    let max_connections = 2;
    let semaphore = Arc::new(Semaphore::new(max_connections));

    loop {
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let (stream1, address1) = listener.accept().await?;
        let (stream2, address2) = listener.accept().await?;

        println!(
            "New connections from {} and {}. Starting chat!",
            &address1, &address2
        );

        tokio::spawn(async move {
            let _p = permit;
            chat(stream1, stream2).await
        });
    }
}

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

async fn chat(stream1: &mut TcpStream, stream2: &mut TcpStream) -> std::io::Result<()> {
    stream1
        .write(
            format!(
                "Hi, you're chatting with {}.\n",
                stream2.peer_addr().unwrap()
            )
            .as_bytes(),
        )
        .await?;
    stream2
        .write(
            format!(
                "Hi, you're chatting with {}.\n",
                stream1.peer_addr().unwrap()
            )
            .as_bytes(),
        )
        .await?;

    let mut buffer1 = [0; 1024];
    let mut buffer2 = [0; 1024];

    loop {
        let future1 = stream1.read(&mut buffer1);
        let future2 = stream2.read(&mut buffer2);

        tokio::select! {
            n = future1 => {
                if let Ok(n) = n {
                    stream2.write(&buffer1[0..n]).await?;
                }
            },
            n = future2 => {
                if let Ok(n) = n {
                    stream1.write(&buffer2[0..n]).await?;
                }
            },
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    let (mut socket1, _) = listener.accept().await?;
    let (mut socket2, _) = listener.accept().await?;

    chat(&mut socket1, &mut socket2).await?;

    Ok(())
}

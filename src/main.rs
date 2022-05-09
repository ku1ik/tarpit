use std::{io, time::Duration, env};
use tokio::{net::{TcpListener, TcpStream}, time::sleep, io::AsyncWriteExt};

async fn handle_connection(mut socket: TcpStream) -> io::Result<()> {
    loop {
        sleep(Duration::from_secs(5)).await;
        let n: u64 = rand::random();
        let s = format!("{:x}\r\n", n);
        socket.write_all(s.as_bytes()).await?;
        socket.flush().await?;
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let bind_addr = env::args().nth(1).unwrap_or("0.0.0.0:2222".to_string());
    let listener = TcpListener::bind(bind_addr).await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        eprintln!("new connection from {}", addr);

        // tokio::spawn(async move { handle_connection(socket).await });

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("write error for {}: {}", addr, e);
            }
        });
    }
}

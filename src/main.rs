use std::{io, time::Duration, env, future::Future};
use tokio::{net::{TcpListener, TcpStream}, time::{sleep, Instant}, io::AsyncWriteExt};
use rand::random;

async fn ssh_handler(mut socket: TcpStream) -> io::Result<()> {
    loop {
        sleep(Duration::from_secs(5)).await;
        let banner = format!("{:x}\r\n", random::<u32>());
        socket.write_all(banner.as_bytes()).await?;
        socket.flush().await?;
    }
}

async fn smtp_handler(mut socket: TcpStream) -> io::Result<()> {
    let reply = format!("220-{:x}\r\n", random::<u32>());
    socket.write_all(reply.as_bytes()).await?;

    loop {
        sleep(Duration::from_secs(5)).await;
        let reply = format!("220-{:x}\r\n", random::<u32>());
        socket.write_all(reply.as_bytes()).await?;
        socket.flush().await?;
    }
}

async fn http_handler(mut socket: TcpStream) -> io::Result<()> {
    socket.write_all("HTTP/1.1 200 OK\r\n".as_bytes()).await?;

    loop {
        sleep(Duration::from_secs(5)).await;
        let header = format!("X-{:x}: {:x}\r\n", random::<u32>(), random::<u32>());
        socket.write_all(header.as_bytes()).await?;
        socket.flush().await?;
    }
}

async fn accept<F, Fut>(listener: TcpListener, handler: F) where
    F: Fn(TcpStream) -> Fut,
    Fut: Future<Output = io::Result<()>> + std::marker::Send + 'static {
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                eprintln!("new connection from {}", addr);
                let h = handler(socket);
                
                tokio::spawn(async move {
                    let now = Instant::now();

                    if let Err(e) = h.await {
                        eprintln!("socket error for {}: {}", addr, e);
                        eprintln!("client {} trapped for {} sec", addr, now.elapsed().as_secs());
                    }
                });
            }

            Err(e) => {
                eprintln!("accept error: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let ssh_bind_addr = env::args().nth(1).unwrap_or("0.0.0.0:2222".to_string());
    let ssh_listener = TcpListener::bind(ssh_bind_addr).await?;
    let ssh = accept(ssh_listener, ssh_handler);

    let smtp_bind_addr = env::args().nth(2).unwrap_or("0.0.0.0:2525".to_string());
    let smtp_listener = TcpListener::bind(smtp_bind_addr).await?;
    let smtp = accept(smtp_listener, smtp_handler);

    let http_bind_addr = env::args().nth(3).unwrap_or("0.0.0.0:8080".to_string());
    let http_listener = TcpListener::bind(http_bind_addr).await?;
    let http = accept(http_listener, http_handler);

    tokio::join!(ssh, smtp, http);

    Ok(())
}

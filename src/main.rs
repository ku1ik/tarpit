use clap::Parser;
use rand::random;
use std::future::Future;
use std::io;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Instant};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Enable SSH tarpit on given listen address
    #[clap(long)]
    ssh: Option<String>,

    /// Enable SMTP tarpit on given listen address
    #[clap(long)]
    smtp: Option<String>,

    /// Enable HTTP tarpit on given listen address
    #[clap(long)]
    http: Option<String>,
}

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

async fn accept<F, Fut>(listener: TcpListener, handler: F)
where
    F: Fn(TcpStream) -> Fut,
    Fut: Future<Output = io::Result<()>> + std::marker::Send + 'static,
{
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                eprintln!("new connection from {}", addr);
                let h = handler(socket);

                tokio::spawn(async move {
                    let now = Instant::now();
                    let e = h.await.unwrap_err();
                    eprintln!("socket error for {}: {}", addr, e);
                    eprintln!(
                        "client {} trapped for {} sec",
                        addr,
                        now.elapsed().as_secs()
                    );
                });
            }

            Err(e) => {
                eprintln!("accept error: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut handles = Vec::new();

    if let Some(ssh_bind_addr) = cli.ssh {
        let ssh_listener = TcpListener::bind(ssh_bind_addr).await?;
        handles.push(tokio::spawn(accept(ssh_listener, ssh_handler)));
    }

    if let Some(smtp_bind_addr) = cli.smtp {
        let smtp_listener = TcpListener::bind(smtp_bind_addr).await?;
        handles.push(tokio::spawn(accept(smtp_listener, smtp_handler)));
    }

    if let Some(http_bind_addr) = cli.http {
        let http_listener = TcpListener::bind(http_bind_addr).await?;
        handles.push(tokio::spawn(accept(http_listener, http_handler)));
    }

    if !handles.is_empty() {
        for handle in handles {
            handle.await?;
        }

        Ok(())
    } else {
        anyhow::bail!("at least one of --ssh/--smtp/--http options needed");
    }
}

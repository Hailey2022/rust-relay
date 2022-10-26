use smol::net::TcpListener;
use smol::net::TcpStream;
use smol::stream::StreamExt;
use smol::prelude::*;

async fn tcp_relay(
    local_addr: std::net::SocketAddr,
    remote_addr: std::net::SocketAddr,
) -> anyhow::Result<()> {
    let local = TcpListener::bind(local_addr).await?;
    let mut incoming = local.incoming();
    loop {
        if let Some(stream) = incoming.next().await {
            let stream = stream?;
            let remote = TcpStream::connect(remote_addr).await?;
            let handle: smol::Task<anyhow::Result<()>> = smolscale::spawn(async move {
                smol::io::copy(remote.clone(), stream.clone())
                    .race(smol::io::copy(stream, remote))
                    .await?;
                Ok(())
            });
            handle.detach()
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let local = match args.get(1) {
        Some(listen) => listen.parse::<std::net::SocketAddr>()?,
        None => "127.0.0.1:9999".parse::<std::net::SocketAddr>()?,
    };
    let remote = match args.get(2) {
        Some(listen) => listen.parse::<std::net::SocketAddr>()?,
        None => "127.0.0.1:5201".parse::<std::net::SocketAddr>()?,
    };
    smol::block_on(async {
        while let Err(e) = tcp_relay(local, remote).await {
            eprintln!("Error: {}", e);
        }
    });
    Ok(())
}
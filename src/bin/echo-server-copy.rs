use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::info;

const ADDR: &str = "127.0.0.1:6100";

//io::copy(&mut reader, &mut file).await?; copies the entire reader into a writer


#[tokio::main]
async fn main() -> io::Result<()> {

    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind(ADDR).await?;

    loop {
        let (mut socket, info) = listener.accept().await?;
        
        // we can also call socket.split(); however, they must be on the same task. If you need more flexibility to other tasks, use into_split

        info!("accepted connectio from: {:?}", info);
        
        // splitting reader and writer as separate handlers so that we dont need to use the same mutable refernece
        // from this socket
        let (mut rd, mut wr) = io::split(socket);


        tokio::spawn(async move {
            wr.write_all(b"hello\r\n").await?;
            wr.write_all(b"world\r\n").await?;

            Ok::<_, io::Error>(()) // just helping the type inferencer to understand the return types
        });

        // reading from the receiver

        let mut buf = vec![0; 128]; // initializing 128 entries of 0

        loop {
            let n = rd.read(&mut buf).await?;

            if n == 0 {
                break; // end of message
            }

            info!("got data from reader: {:?}", &buf[..n]);
        }

        // Ok(())
    }

    
}
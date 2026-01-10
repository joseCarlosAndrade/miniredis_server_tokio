use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{info, error};

const ADDR: &str = "127.0.0.1:6100";

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(ADDR).await?;
    tracing_subscriber::fmt::init();
    
    loop {
    
        let (mut socket, info) = listener.accept().await?;
        info!("accepted connection from: {:?}", info);

        tokio::spawn(async move {
            let mut buf = vec![0; 1024]; // buffer that starts with 1024 entries of 0
            // its usually better to define buffers using heap allocations like Box or Vec (as we did here)
            // bc this means that the "task enum" would only have to store its pointer, and not the whole xxxxx bytes of stack size
            // so, in conclusion, its usually better to avoid huge stack allocations like let buffer = [0; 10000];

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        info!("client closed connection");
                        return;
                    },
                    Ok(n) => {
                        // received n bytes from client
                        // to convert bytes to utf8 text: let text = std::str::from_utf8(&buf[..n]).unwrap();
                        info!("received {} bytes from the client\nmessage: \n{:#?}", n, &buf[..n]); //using pretty print
                        // note: if we try to use bytes[..n], the compiler tries to pass the whole n chunk data into that function
                        // which is not possible since we dont know n at compile time, so it throws an error
                        // to do this, we pass a reference to that data &buf[..] which is Sizeded (has a fixed size)
                        // by doing this, we're only passing a pointer and a length to the compiler, and both sizes are known
                        // in compile time

                        // writing it back to the socket
                        if socket.write_all(&buf[..n]).await.is_err() {
                            error!("error sending message back to client");
                        }
                    },
                    Err(e) => {
                        error!("error reading data from client. terminating connection: {:?}", e);
                        // drop(socket); // todo: how to deal with errors? should i close the connection? do rust already do this when it goes out of scope?
                        // answer: yes, when a TcpStream goes out of scope, the Drop implementation is triggered
                        // it cleans resources and closes the connection

                        return;
                    }
                }
            }
        });
    }

}
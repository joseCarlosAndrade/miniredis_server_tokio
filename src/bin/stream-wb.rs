
//! # Websocket streams implementation
//! 
//! using tokio_tungstenite to accept streams from webscoekt connection and 
//! StreamExt to read and write to that stream

use tokio::{net::TcpListener};
use tokio_tungstenite::{accept_async, tungstenite::Message}; // deals with websocket protocol
use futures_util::StreamExt; // provides utilities like next(), map(), filter() on stream data
use tracing::{error, info, warn};

const ADDR: &str = "127.0.0.1:6100";

#[tokio::main]
async fn main()  {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind(ADDR).await.unwrap();

    while let Ok((stream, info)) = listener.accept().await {
        tokio::spawn(async move {   
            info!("accepted connection from: {:?}", info);

            // performs a handshake, turning a generic tcp connection into a websocket stream
            // todo: keep checking the clients health with ping pong so we dont keep storing dead connections
            // in case of abrubt disconnect
            let mut ws_stream = accept_async(stream).await.unwrap();

            // this stream implements both Stream (reding)and Sink (writing)
            // let (mut write_half, mut read_half) = ws_stream.split();

            while let Some(msg) = ws_stream.next().await {
                // do something with the message here
                match msg {
                    Ok(m) => {
                        

                        match m {
                            Message::Text(t) => {
                                info!("message got: {}", t);
                            },
                            Message::Close(c) => {
                                if c.as_ref().is_none() {
                                    warn!("client sent close without reason");
                                    return;
                                }
                                warn!("client sent close with reason: {}", c.unwrap().reason);
                                return;
                            },
                            _ => {return;}
                        }
                    },
                    Err(e) => {
                        error!("error getting message : {:?}", e);
                        return;
                    }
                }
            }
        });
    }
}
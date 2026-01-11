
//! # Websocket streams implementation
//! 
//! using tokio_tungstenite to accept streams from webscoekt connection and 
//! StreamExt to read and write to that stream

use tokio::{net::TcpListener};
use tokio_tungstenite::{accept_async, tungstenite::Message}; // deals with websocket protocol
use futures_util::{StreamExt, SinkExt}; // provides utilities like next(), map(), filter() on stream data
use tracing::{error, info, warn, debug};
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields="camelCase")] // renames all fields to snake case (for incoming messages). it also uses the field "type" to decide which enum vaiant to use
enum FromClientMessage {
    StartAll {
        // #[serde(rename = "nameS")] // rename a speciifc field. not necessary here bc we use both rename_all and rename_all_fields
        name_s : String
    },
    Text { content : String},
    Stop,
    Chat { message : String},
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields="camelCase")] // renames all fields to camelCase
enum FromServerMessage {
    ReplyMessage { 
        // #[serde(rename = "contentMessage")]
        content_message : String
    },
    Stop {fields : String},
}

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
            let (mut write_half, mut read_half) = ws_stream.split();

            while let Some(msg) = read_half.next().await {
                // do something with the message here
                match msg {
                    Ok(m) => {
                        // tungstenite has helpers to avoid this matching:
                        // if m.is_text() { let str = m.to_text().unwrap(); }
                        // if m.is_close() {..}


                        match m {
                            Message::Text(t) => {
                                info!("text got: {}", t);

                                // TODO:
                                // figure out a way of sending json messages (maybe json wont work)
                                // we gotta be able to send several types of messages
                                // we gotta be able to parse several types of messages
                                // package serde and serde json

                                // treating json messages:
                                match serde_json::from_str::<FromClientMessage>(&t) { // deserialize (turns json to type)
                                    Ok(client_msg) => {
                                        match client_msg {
                                            FromClientMessage::StartAll { name_s } => {
                                                debug!("received start message: {}", name_s);

                                                // serializing data
                                                let raw = FromServerMessage::ReplyMessage { content_message: name_s };
                                                let reply = serde_json::to_string(&raw).unwrap();
                                                info!("SENDING TO CLIENT: {}", reply);
                                                
                                                // must use futures_util::SinkExt
                                                let data = Message::Text(reply);
                                                write_half.send(data).await.unwrap();

                                                // works when receiving {"type" : "start_all", "name" : "jc"}
                                            },
                                            FromClientMessage::Stop => {

                                            },
                                            FromClientMessage::Chat { message } => {

                                            },
                                            FromClientMessage::Text { content } => {

                                            }
                                        }
                                    },
                                    Err(e) => {
                                        error!("unknown message format: {:?}", e);
                                    }
                                }
                            },
                            Message::Binary(b) => {
                                info!("binary received: {:#?}", b);
                            },
                            Message::Close(frame) => {
                                if frame.as_ref().is_none() {
                                    warn!("client sent close without reason");
                                    return;
                                }
                                info!("client sent close with reason: {}. code: {}", frame.as_ref().unwrap().reason, frame.as_ref().unwrap().code);
                                return;
                            },
                            Message::Ping(payload) => {
                                // tungstenite usually handles ping automatically
                                debug!("received ping. payload: {:?}", payload);
                            },
                            Message::Pong(_) => {
                                debug!("received Pong (client is alive)");
                            },
                            _ => {return;}
                        }
                    },
                    Err(e) => { 
                        // it may be:
                        // ConnectionReset: Client's internet died or they force-closed.
                        // Protocol:	Client sent invalid WebSocket data (hacking or bug).
                        // Capacity:	Client sent a message too big for your buffer
                        //Tls:	Encryption error (if using wss://).
                        error!("error getting message : {:?}", e);
                        return;
                    }
                }
            }
        });
    }
}
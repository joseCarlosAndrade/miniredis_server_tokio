//! # Messing with networking in rust - Server code
//! 
//! This is really fun
//! 
//! how to open this doc in html:
//! ```
//! cargo doc --open --document-private-items 
//! ```
//! 
//! this code opens a socket on 127.0.0.1:6000 that accepts redis protocol

use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use std::{collections::HashMap};
use bytes::Bytes;
use std::sync::{Arc, Mutex};
// arc is used to shared references of this hashmap accross concurrent tasks
// term handle is used to reference a value that provides access to some shared state
// in our case, the db handle is an access to the shared hashmap

use tracing::{info, debug};


type Db = Arc<Mutex<HashMap<String, Bytes>>>;

const ADDR : &str = "127.0.0.1:6000";

// note : use info! or debug! for logging

#[tokio::main]
async fn main() {
    
    let listener = TcpListener::bind(ADDR).await.unwrap();

    // println!("listening");
    info!("listening..");
    

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop  {
        let (socket, info) = listener.accept().await.unwrap();

        println!("{:?}", info);
        // process(socket).await; // single thread version. it process one requests and blocks until its fully written

        // we're cloning the handle to the hm, not the hm itself
        let db = db.clone(); // each iteration clones the handle to the hashmap so it's not moved

        let _ = tokio::spawn(async move { // move is necessaru bc the task may live longer than our main function. hence move moves all variables to the task (in this case, socket. if there was no move, when this main finishes socket would be dropped but the closure would still have a reference to it)
            
            
            process(socket, db).await;
            
        });
    }
}

async fn process(socket: TcpStream, db : Db) {

    let mut connection = Connection::new(socket);

    // let mut db: HashMap<String, Vec<u8>> = HashMap::new();

   

    // redis protocol: passes data through frames. the mini-redis library abstract this protocol with read_frame
    // if the client sends whatever else, it gets an error
    loop {
        // let db = db.clone();
    
        match connection.read_frame().await {
            Err(e) => { // protocol error: invalid frame
                println!("invalid data: must follow redis frame protocol: {:?}", e);
                return;
            },
            
            Ok(conn) => { // data went ok, but it may be none
                if let Some(frame) = conn {
                    println!("got: {:?}", frame);
            
                    // let response = Frame::Error("this sevice was not yet implemented".to_string());
                    // connection.write_frame(&response).await.unwrap();
                    process_frame(frame, &mut connection, db.clone()).await.unwrap();

                    println!("{:?}", db); // pritntln does not move the values!
                    println!("\n====== finished this operation ======\n");
                }
            }
        }

    }
    
}

/// # Process frame
/// 
/// process a single frame and writes it back to the connection
/// 
/// todo:
/// - it might be unecessary to pass connection here. we could just return the response the server should send back
/// as the Ok in the Result type and leave this responsibility to the process function
async fn process_frame(frame : Frame, connection : &mut Connection, db : Db) -> Result<(), String> {
    
    use mini_redis::Command::{self, Get, Set};

    let response : Frame = 
        match Command::from_frame(frame) { // tries to disasseble the frame received as a command
            Ok(command) => {
                match command { // if it is a command, check which one is it
                    Set(cmd) => {
                        println!("doing set... key: {} value: {:?}", cmd.key(), cmd.value());
            
                        // do stuff here set
                        let mut db = db.lock().unwrap();
                       db.insert(cmd.key().to_string(), cmd.value().clone()); // stored as Bytes (cloning in Bytes is cheaper)
            
                        Frame::Simple("OK".to_string()) // response will be OK
                    }
                    Get(cmd) => {
                        println!("doing get... key: {}" , cmd.key());
            
                        // do stuff here
                        let db = db.lock().unwrap(); // locks and unlocks when it goes out of scope
                        if let Some(value) = db.get(cmd.key()) {
                            println!("found: {:?}", value);

                            Frame::Bulk(value.clone()) // response will be value
                        } else {
                            println!("no key found");

                            Frame::Null
                        }
                    }
                    cmd => { // rest, unimplemented yet
                        println!("unimplemented command: {:?}", cmd);
                        
                        Frame::Null // null for yet
                    },
                }
            },
            Err(_) => {
                return Err("could not disasemble into command".into()); // error, could not even disassemble. just return err to the main function
            },
        }; // this whole block evaluates to the response variable


    if let Err(e) = connection.write_frame(&response).await { // tries to write back to the client
        println!("error sending response back to client: {:?}", e);

        return Err(format!("could not send response back to client: {e}"));
    }

    Ok(())
}
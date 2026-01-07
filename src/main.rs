//! # Messing with networking in rust
//! 
//! This is really fun
//! 
//! how to open this doc in html:
//! ```
//! cargo doc --open --document-private-items 
//! ```

use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};

const ADDR : &str = "127.0.0.1:6000";

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(ADDR).await.unwrap();

    loop  {
        let (socket, info) = listener.accept().await.unwrap();

        println!("{:?}", info);
        // process(socket).await; // single thread version. it process one requests and blocks until its fully written

        tokio::spawn(async move { // move is necessaru bc the task may live longer than our main function. hence move moves all variables to the task (in this case, socket. if there was no move, when this main finishes socket would be dropped but the closure would still have a reference to it)
            process(socket).await;
        });

    }
}

async fn process(socket: TcpStream) {

    let mut connection = Connection::new(socket);

    // redis protocol: passes data through frames. the mini-redis library abstract this protocol with read_frame
    // if the client sends whatever else, it gets an error
    loop {
    
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
                    process_frame(frame, &mut connection).await.unwrap();
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
async fn process_frame(frame : Frame, connection : &mut Connection) -> Result<(), String> {
    
    use mini_redis::Command::{self, Get, Set};

    let response : Frame = 
        match Command::from_frame(frame) { // tries to disasseble the frame received as a command
            Ok(command) => {
                match command { // if it is a command, check which one is it
                    Set(cmd) => {
                        println!("doing set... key: {} value: {:?}", cmd.key(), cmd.value());
            
                        // do stuff here set
            
                        Frame::Simple("OK".to_string()) // response will be OK
                    }
                    Get(cmd) => {
                        println!("doing get... key: {}" , cmd.key());
            
                        // do stuff here
            
                        Frame::Bulk("value".into()) // response will be value
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
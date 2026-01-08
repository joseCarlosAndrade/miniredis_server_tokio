//! client code
//! 
//! this client connects to the server using socket tcp and redis protocol
//! implementing using channels
//! one task will be responsible for the client management 
//! the others will be per-connection and will communicate with the client management through message passing
//! the channel will act as a buffer for commands
//! 
//! mpsc -> many values can be sent
//! oneshot -> single producer, single consumer, single value can be sent
//! broadcast -> multi producer, multi-consumer, many values can be sent, each receive sees every value
//! watch -> multi producer, multi consumer, many values can be sent but no history is kept. receivers oly see the most recent values

use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};
use mini_redis::{client};
use tracing::{info, debug};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key : String,
        resp : Responder<Option<Bytes>>,
    },
    Set {
        key : String,
        val : Bytes,
        resp : Responder<()>
    }
}

// #[derive(Debug)] // while i could define a custom response, we'll stick to the pre defined miniredis implementation
// enum Response {
//     Get {
//         val :  Bytes,
//     },
//     Set {
//         status: String,
//     }
// }

const ADDR : &str= "127.0.0.1:6000";

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Command>(32); // channel with capacity of 32
    // the sender and receiver tx and rx may be moved to different tasks
    // sender are clone to each new task. NOT: FOR MPSC THE RECEIVER CANNOT BE CLONED
    // mpsc is used to emmit commands from tasks to the client manager

    let tx_clone = tx.clone();

    // to manage the response (client manager -> tasks) we'll use a oneshot, sending the Sender with the command
    // this way the manager may respond to the task that emmited the command

    let manager = tokio::spawn(async move { // move must move the rx to here
        let mut client = client::connect(ADDR).await.unwrap();

        while let Some(cmd) = rx.recv().await { // while it receives something. when all senders are dropped, this will return None
            match cmd {
                Command::Get{key, resp} => {
                    info!("received get command from channel");
                    debug!("key: {}", key);

                    let res = client.get(&key).await;
                    _ = resp.send(res); // send on oneshot does not need await
                    
                }
                Command::Set { key, val, resp} => {
                    info!("received set command from channel");
                    debug!("key: {}, val: {:?}", key, val);

                    let r =client.set(&key, val).await;
                    _ = resp.send(r);
                }
            }
        }
    });

    let t1 = tokio::spawn(async move {
        let (resp, rx) = oneshot::channel(); // oneshot is default 1

        let cmd = Command::Get { 
            key: "aa".to_string() ,
            resp : resp,
        };

        tx_clone.send(cmd).await.unwrap();

        let r = rx.await;
        println!("sending get: GOT: {:?}", r);
    });

    let t2 = tokio::spawn(async move {
        let (resp, rx) = oneshot::channel(); // oneshot is default 1

        let cmd = Command::Set{
            key: "aa".to_string(),
            val : "value".into(),
            resp : resp,
        };

        tx.send(cmd).await.unwrap();

        let r = rx.await;
        println!("sending set: GOT: {:?}", r);
    });

    
    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();

}
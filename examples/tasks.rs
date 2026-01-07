

#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async { // handle allows us to interact with the task
        // do stuff here
        "return something" // string for example
    });
    let out = handle.await.unwrap();
    println!("got {}", out);
}
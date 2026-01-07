use mini_redis::{client, Result};

const ADDR : &str = "127.0.0.1:6000";

#[tokio::main]
async fn main() -> Result<()> {
    println!("starting hello-redis example");

    let mut client = client::connect(ADDR).await?;

    client.set("key", "values".into()).await?;

    let result = client.get("key").await?;

    println!("got value: {:?}", result);

    Ok(())
}

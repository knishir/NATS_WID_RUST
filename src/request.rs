use std::{env, str::from_utf8, time::Duration};

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let nats_url = env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());

    let client = async_nats::connect(nats_url).await?;

    let response = client.request("TALWIINDER.sue", "".into()).await?;
    println!("got a response: {:?}", from_utf8(&response.payload)?);

    let response = client.request("TALWIINDER.john", "".into()).await?;
    println!("got a response: {:?}", from_utf8(&response.payload)?);

    let response = tokio::time::timeout(
        Duration::from_millis(500),
        client.request("TALWIINDER.bob", "".into()),
    )
    .await??;
    println!("got a response: {:?}", from_utf8(&response.payload)?);

    Ok(())
}

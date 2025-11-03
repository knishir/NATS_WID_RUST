use futures::StreamExt;
use std::env;
use std::str::from_utf8; // FOR CONVERTING BYTES TO STRING

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    // LOOKS FOR NATS_URL IN ENVIRONMENT VARIABLES IF NOT THEN USES DEFAULT
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "localhost:4222".to_string());

    let client = async_nats::connect(nats_url).await?;
    client.publish("Barak.*", "HELLO FROM RUST!".into()).await?;

    let mut subscription = client.subscribe("Barak.*").await?.take(3);

    for subject in ["Barak.Obama", "Barak.Michelle", "Barak.Sasha"] {
        client.publish(subject, "HELLO FROM RUST!".into()).await?;
    }

    while let Some(message) = subscription.next().await {
        match from_utf8(message.payload.as_ref()) {
            Ok(text) => println!("{} received on {}", text, message.subject),
            Err(_) => println!(
                "<binary> received on {} ({} bytes)",
                message.subject,
                message.payload.len()
            ),
        }
    }

    Ok(())
}

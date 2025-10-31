use futures::StreamExt;
use std::env;

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let nats_url = env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());

    let client = async_nats::connect(nats_url).await?;

    let mut requests = client.subscribe("TALWIINDER.*").await.unwrap();

    while let Some(request) = requests.next().await {
        if let Some(reply) = request.reply {
            let name = &request.subject[6..];
            client
                .publish(reply, format!("hello, {}", name).into())
                .await?;
        }
    }

    Ok(())
}

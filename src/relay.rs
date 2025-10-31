use futures::StreamExt;
use std::env;
use std::str::from_utf8; 
use std::time::Duration;


#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let nats_url = env::var("NATS_URL")
    .unwrap_or_else(|_| "nats://localhost:4222".to_string());

    let client = async_nats::connect(nats_url).await?;
    
    let mut requests = client.subscribe("MICHALE.*").await.unwrap();

    tokio::spawn({
        let client = client.clone();
        async move {
            while let Some(request) = requests.next()
            .await {
                if let Some(reply) = request.reply {
                    let name = &request.subject[6..];
                    client.publish(reply, format!("HEY, {}", name).into()).await?;
                }
            }
            Ok::<(), async_nats::Error>(())
        }
    });

    let response = client.request("MICHALE.JORDON", "".into()).await?;
    println!("GOT A RESPONSE: {:?}", from_utf8(&response.payload)?);
    
    let response = client.request("MICHALE.JAIGIRI","".into()).await?;
    println!("RECEIVED YOUR RESPONSE: {:?}", from_utf8(&response.payload)?);

    let response = tokio::time::timeout(
        Duration::from_millis(500),client.request("MICHALE.UNKNOWN","".into()),)
        .await??;
    println!("GOT A RESPONSE: {:?}", from_utf8(&response.payload)?);
    Ok(())

}
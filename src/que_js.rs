use async_nats::jetstream::{self,stream};
use futures::{StreamExt, TryStreamExt};


#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let client = async_nats::connect(nats_url).await?;


    let jetstream = jetstream::new(client);

    let mut stream = jetstream.create_stream(stream::Config {
        name: "EVENTS".to_string(),
        retention: stream::RetentionPolicy::WorkQueue,
        subjects: vec!["events.>".to_string()],
        ..Default::default()
}).await?;
println!("Created the stream");

jetstream.publish("events.us.page_loaded","".into()).await?.await?;
jetstream.publish("events.us.mouse_clicked","".into()).await?.await?;
jetstream.publish("events.us.input_focused","".into()).await?.await?;
println!("Published 3 messages");


println!("Stream info without any consumer: {:#?}",stream.info().await?);

let consumer = stream.create_consumer(jetstream::consumer::pull::Config { durable_name: Some("processor-1".to_string()), ..Default::default() }).await?;

let mut messages = consumer.fetch().max_messages(3).messages().await?;
while let Some(message) = messages.next().await {
    message?.ack().await?;
}
    println!("Stream info with one consumer: {:?}", stream.info().await?);


let err = stream.create_consumer(jetstream::consumer::pull::Config {
    durable_name: Some("processor-2".to_string()),
    ..Default::default()
}).await.expect_err("fail to create ovrrlapping consumer");
println!("Create an overlapping consumer: {:?}", err);


stream.delete_consumer("processor-1").await?;
let result = stream.create_consumer(jetstream::consumer::pull::Config { durable_name: Some("processor-2".to_string()), ..Default::default() }).await;
println!("Created the new consumer? {}", result.is_ok());
stream.delete_consumer("processor-2").await?;


   let us_consumer = stream.create_consumer(jetstream::consumer::pull::Config {
    durable_name: Some("processor-us".to_string()),
    filter_subject: "events.us.>".to_string(), ..Default::default()
        })
        .await?;

    let eu_consumer =stream.create_consumer(jetstream::consumer::pull::Config {
        durable_name: Some("processor-eu".to_string()),
        filter_subject: "events.eu.>".to_string(), ..Default::default()    
    }).await?;

    jetstream.publish("events.eu.mouse_clicked","".into()).await?.await?;
    jetstream.publish("events.us.page_loaded","".into()).await?.await?;
    jetstream.publish("events.us.input_focused","".into()).await?.await?;
    jetstream.publish("events.eu.page_loaded","".into()).await?.await?;
    println!("Published 4 messages to different subjects");


    let mut us_messages = us_consumer.fetch().max_messages(2).messages().await?;
    let mut eu_messages = eu_consumer.fetch().max_messages(2).messages().await?;

    while let Some(message) = us_messages.try_next().await? {
        println!("us consumer got: {}", message.subject);
        message.ack().await?;
    }

    while let Some(message) = eu_messages.try_next().await? {
        println!("eu consumer got: {}", message.subject);
        message.ack().await?;
    }
    Ok(())

}

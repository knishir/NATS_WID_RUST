use async_nats::jetstream::{self, stream};
use futures::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    // Step 1: Connect to NATS server
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let client = async_nats::connect(nats_url).await?;

    // Step 2: Initialize JetStream context
    let jetstream = jetstream::new(client);

    // Step 3: Create a new stream (unique name + unique subjects)
    let mut stream = jetstream
        .create_stream(jetstream::stream::Config {
            name: "EVENTS_INT_DEMO".to_string(), // ✅ Changed stream name
            retention: stream::RetentionPolicy::Interest,
            subjects: vec!["int_demo_events.>".to_string()], // ✅ Changed subject prefix
            ..Default::default()
        })
        .await?;
    println!("✅ Created stream: EVENTS_INT_DEMO");

    // Step 4: Publish a few demo events
    jetstream
        .publish("int_demo_events.page_loaded", "".into())
        .await?
        .await?;
    jetstream
        .publish("int_demo_events.mouse_clicked", "".into())
        .await?
        .await?;
    let ack = jetstream
        .publish("int_demo_events.input_focused", "".into())
        .await?
        .await?;

    println!("🟢 Last message sequence: {}", ack.sequence);

    // Step 5: Print stream info (before consumers)
    println!("\n# Stream info without any consumers:");
    println!("{:#?}", stream.info().await?);

    // Step 6: Create first consumer
    let first_consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("processor-1".to_string()),
            filter_subject: "int_demo_events.>".to_string(),
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            ..Default::default()
        })
        .await?;
    println!("✅ Created consumer: processor-1");

    // Step 7: Publish more messages
    jetstream
        .publish("int_demo_events.mouse_clicked", "".into())
        .await?
        .await?;
    jetstream
        .publish("int_demo_events.input_focused", "".into())
        .await?
        .await?;

    println!("\n# Stream info with one consumer:");
    println!("{:#?}", stream.info().await?);

    // Step 8: Fetch and ACK messages for first consumer
    let mut messages = first_consumer.fetch().max_messages(2).messages().await?;
    while let Some(message) = messages.try_next().await? {
        message.double_ack().await?;
    }
    println!("\n# Stream info with acked messages (processor-1):");
    println!("{:#?}", stream.info().await?);

    // Step 9: Create second consumer
    let second_consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("processor-2".to_string()),
            filter_subject: "int_demo_events.>".to_string(),
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            ..Default::default()
        })
        .await?;
    println!("✅ Created consumer: processor-2");

    // Step 10: Publish more messages for second consumer
    jetstream
        .publish("int_demo_events.input_focused", "".into())
        .await?
        .await?;
    jetstream
        .publish("int_demo_events.mouse_clicked", "".into())
        .await?
        .await?;

    // Step 11: Fetch and ACK messages for second consumer
    let messages = second_consumer
        .fetch()
        .max_messages(2)
        .messages()
        .await?
        .try_collect::<Vec<jetstream::Message>>()
        .await?;

    let message1 = messages.first().unwrap();
    let message2 = messages.get(1).unwrap();

    println!(
        "📩 Message sequences received: {} and {}",
        message1.info().unwrap().stream_sequence,
        message2.info().unwrap().stream_sequence
    );

    message1.ack().await?;
    message2.double_ack().await?;

    println!("\n# Stream info with two consumers (partial ack):");
    println!("{:#?}", stream.info().await?);

    // Step 12: Acknowledge remaining messages for both consumers
    first_consumer
        .fetch()
        .max_messages(2)
        .messages()
        .await?
        .try_for_each(|message| async move { message.double_ack().await })
        .await?;

    println!("\n# Stream info with both consumers acked:");
    println!("{:#?}", stream.info().await?);

    // Step 13: Create a third consumer (different subject filter)
    stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("processor-3".to_string()),
            filter_subject: "int_demo_events.mouse_clicked".to_string(),
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            ..Default::default()
        })
        .await?;
    println!("✅ Created consumer: processor-3");

    // Step 14: Publish and test with third consumer
    jetstream
        .publish("int_demo_events.mouse_clicked", "".into())
        .await?
        .await?;

    first_consumer
        .fetch()
        .max_messages(1)
        .messages()
        .await?
        .try_next()
        .await?
        .expect("should have a message")
        .ack_with(jetstream::AckKind::Term)
        .await?;

    second_consumer
        .fetch()
        .max_messages(1)
        .messages()
        .await?
        .try_next()
        .await?
        .expect("should have message")
        .double_ack()
        .await?;

    println!(
        "\n# Stream info with three consumers (final state):\n{:#?}",
        stream.info().await?
    );

    Ok(())
}

use anyhow::Result;
use futures_util::pin_mut;
use std::env;
use sushii_processor::events::get_events;
use sushii_processor::twilight_model::gateway::event::DispatchEvent;
use tokio::stream::StreamExt;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let redis_url = env::var("REDIS_URL").expect("Expected REDIS_URL in the environment");

    let stream = get_events(redis_url);

    tracing::info!("Connected to redis");

    pin_mut!(stream);

    while let Some(e) = stream.next().await {
        let (shard_id, event) = match e {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Error reading event: {}", e);

                continue;
            }
        };

        tokio::spawn(handle_event(shard_id, event));
    }
}

async fn handle_event(
    shard_id: u64,
    event: DispatchEvent,
    // http: HttpClient,
) -> Result<()> {
    let event_name = event.kind().name().unwrap();

    tracing::info!("Received event on shard {}: {}", shard_id, event_name);

    match event {
        DispatchEvent::MessageCreate(msg) if msg.content == "!ping" => {
            // http.create_message(msg.channel_id).content("Pong!")?.await?;
        }
        // Other events here...
        _ => {}
    }

    Ok(())
}

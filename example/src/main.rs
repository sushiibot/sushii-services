use std::env;
use sushii_processor::events::get_events;
use tokio::stream::StreamExt;
use futures_util::pin_mut;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let redis_url = env::var("REDIS_URL").expect("Expected REDIS_URL in the environment");

    let stream = get_events(redis_url);

    tracing::info!("Connected to redis");

    pin_mut!(stream);

    while let Some(event) = stream.next().await {
        let event = match event {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Error reading event: {}", e);

                continue;
            }
        };

        let event_name = event.kind().name().unwrap();

        tracing::info!("Received event: {}", event_name);
    }
}

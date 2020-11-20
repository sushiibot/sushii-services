use async_stream::try_stream;
use darkredis::Connection;
use serde::de::DeserializeSeed;
use serde_json::Deserializer;
use tokio::stream::Stream;
use twilight_model::gateway::event::DispatchEvent;
use twilight_model::gateway::event::DispatchEventWithTypeDeserializer;

use crate::error::Result;

pub fn get_events(redis_addr: String) -> impl Stream<Item = Result<DispatchEvent>> {
    try_stream! {
        let mut conn = Connection::connect(redis_addr).await?;

        loop {
            if let Some((_list, event)) = conn.blpop(&["events"], 0).await? {
                let event_str = String::from_utf8_lossy(&event);

                if event_str == "event" {
                    break;
                }

                let split_pos = match event_str.find(',') {
                    Some(pos) => pos,
                    None => {
                        tracing::warn!("Event has no , to split: {}", event_str);
                        continue;
                    }
                };

                let event_name = &event_str[..split_pos];
                let event_json_str = &event_str[split_pos + 1..];

                tracing::info!("Event json string: {}", event_json_str);

                // Create deserializer with the event name
                let de = DispatchEventWithTypeDeserializer::new(event_name);

                let mut json_deserializer = Deserializer::from_str(event_json_str);
                let event = de.deserialize(&mut json_deserializer)?;

                tracing::info!("blpop: {}", event_name);

                yield event;
            }
        }
    }
}

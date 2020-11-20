use async_stream::try_stream;
use darkredis::Connection;
use serde::de::DeserializeSeed;
use serde_json::Deserializer;
use tokio::stream::Stream;
use twilight_model::gateway::event::DispatchEvent;
use twilight_model::gateway::event::DispatchEventWithTypeDeserializer;

use crate::error::Result;

fn parse_event_str<'a>(s: &'a str) -> Option<(u64, &'a str, &'a str)> {
    let split_pos_1 = s.find(',')?;
    let shard_id = s[..split_pos_1].parse::<u64>().ok()?;

    // Find split pos for second half
    let split_pos_2 = s[split_pos_1 + 1..].find(',')?;

    let event_name = &s[split_pos_1 + 1..split_pos_2];
    let event_json_str = &s[split_pos_2 + 1..];

    Some((shard_id, event_name, event_json_str))
}

pub fn get_events(redis_addr: String) -> impl Stream<Item = Result<(u64, DispatchEvent)>> {
    try_stream! {
        let mut conn = Connection::connect(redis_addr).await?;

        loop {
            if let Some((_list, event)) = conn.blpop(&["events"], 0).await? {
                let event_str = String::from_utf8_lossy(&event);

                if event_str == "event" {
                    break;
                }

                // Use a custom struct in a common models crate instead of all this 
                let (shard_id, event_name, event_json_str) = match parse_event_str(&event_str) {
                    Some(data) => data,
                    None => {
                        tracing::warn!("Failed to parse event string: {}", event_str);
                        continue;
                    }
                };

                tracing::info!("Event json string: {}", event_json_str);

                // Create deserializer with the event name
                let de = DispatchEventWithTypeDeserializer::new(event_name);

                let mut json_deserializer = Deserializer::from_str(event_json_str);
                let event = de.deserialize(&mut json_deserializer)?;

                tracing::info!("blpop: {}", event_name);

                yield (shard_id, event);
            }
        }
    }
}

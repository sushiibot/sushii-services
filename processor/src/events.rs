use async_stream::try_stream;
use darkredis::Connection;
use serde::de::DeserializeSeed;
use serde_json::Deserializer;
use tokio::stream::Stream;
use twilight_model::gateway::event::gateway::GatewayEvent;
use twilight_model::gateway::event::DispatchEvent;
use twilight_model::gateway::event::gateway::GatewayEventDeserializer;

use crate::error::Result;

pub fn get_events(redis_addr: String) -> impl Stream<Item = Result<DispatchEvent>> {
    try_stream! {
        let mut conn = Connection::connect(redis_addr).await?;

        loop {
            if let Some((_list, event)) = conn.blpop(&["events"], 0).await? {
                let event_str = String::from_utf8_lossy(&event);

                let de = match GatewayEventDeserializer::from_json(&event_str) {
                    Some(d) => d,
                    None => {
                        tracing::warn!("Failed to create create gateway event deserializer: {}", event_str);
                        continue;
                    }
                };

                let mut json_deserializer = Deserializer::from_str(&event_str);
                let gateway_event = match de.deserialize(&mut json_deserializer) {
                    Ok(event) => event,
                    Err(e) => {
                        tracing::warn!("Failed to deserialize event: {}", e);
                        continue;
                    }
                };

                let dispatch_event = match gateway_event {
                    // (sequence number, event) don't need seq
                    GatewayEvent::Dispatch(_, dispatch_event) => dispatch_event,
                    // Heartbeat(u64), HeartbeatAck, Hello(u64), InvalidateSession(bool), Reconnect,
                    _ => continue,
                };

                // Not sure if I should return a Box<DispatchEvent> or move it
                // back to the stack like this
                yield *dispatch_event;
            }
        }
    }
}

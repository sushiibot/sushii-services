use serde::{Deserialize, Serialize};
use twilight_model::gateway::event::DispatchEvent;
use twilight_model::gateway::event::EventType;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub shard_id: u64,
    #[serde(flatten)]
    pub dispatch_event: DispatchEventWrapper,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DispatchEventWrapper {
    pub event_type: EventType,
    pub event: DispatchEvent,
}

impl Event {
    pub fn new(shard_id: u64, event_type: EventType, event: DispatchEvent) -> Self {
        Self {
            shard_id,
            dispatch_event: DispatchEventWrapper {
                event_type,
                event,
            }
        }
    }
}

use std::convert::TryFrom;
use std::fmt;

use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use twilight_model::gateway::event::DispatchEventWithTypeDeserializer;

// I have literally no idea what I'm doing here but it somehow works 🤔
impl<'de> Deserialize<'de> for DispatchEventWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            EventType,
            Event,
        };

        struct DispatchEventVisitor;

        impl<'de> Visitor<'de> for DispatchEventVisitor {
            type Value = DispatchEventWrapper;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct DispatchEventWrapper")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<DispatchEventWrapper, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let event_type = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let event_type_seed = DispatchEventWithTypeDeserializer::new(event_type);

                let event = seq
                    .next_element_seed(event_type_seed)?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                Ok(DispatchEventWrapper {
                    event_type: EventType::try_from(event_type).map_err(de::Error::custom)?,
                    event,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<DispatchEventWrapper, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut event_type = None;
                let mut event = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::EventType => {
                            if event_type.is_some() {
                                return Err(de::Error::duplicate_field("event_type"));
                            }
                            event_type = Some(map.next_value()?);
                        }
                        Field::Event if event_type.is_some() => {
                            if event.is_some() {
                                return Err(de::Error::duplicate_field("event"));
                            }

                            let event_type_seed =
                                DispatchEventWithTypeDeserializer::new(event_type.unwrap());
                            event = Some(map.next_value_seed(event_type_seed)?);
                        }
                        _ => return Err(de::Error::custom("Event parsed before EventType"))
                    }
                }
                let event_type =
                    event_type.ok_or_else(|| de::Error::missing_field("event_type"))?;

                let event = event.ok_or_else(|| de::Error::missing_field("event"))?;

                Ok(DispatchEventWrapper {
                    event_type: EventType::try_from(event_type).map_err(de::Error::custom)?,
                    event,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["event_type", "event"];
        deserializer.deserialize_struct("DispatchEvent", FIELDS, DispatchEventVisitor)
    }
}

#[test]
fn deserializes_event() {
    let s = "{\"shard_id\":0,\"event_type\":\"MESSAGE_CREATE\",\"event\":{\"activity\":null,\"application\":null,\"attachments\":[],\"author\":{\"avatar\":\"afaddec4029eafd36e30fb62efe7bfad\",\"bot\":false,\"discriminator\":\"7080\",\"email\":null,\"flags\":null,\"id\":\"150443906511667200\",\"locale\":null,\"mfa_enabled\":null,\"username\":\"tzuwy\",\"premium_type\":null,\"public_flags\":131584,\"system\":null,\"verified\":null},\"channel_id\":\"749822555019280434\",\"content\":\"test message\",\"edited_timestamp\":null,\"embeds\":[],\"flags\":0,\"guild_id\":\"167058919611564043\",\"id\":\"779206031192096789\",\"type\":0,\"member\":{\"deaf\":false,\"joined_at\":\"2016-04-05T23:52:30.292000+00:00\",\"mute\":false,\"nick\":null,\"roles\":[\"194255298452652032\",\"194255422654251009\",\"194648344553979904\",\"167061672631074817\"]},\"mention_channels\":[],\"mention_everyone\":false,\"mention_roles\":[],\"mentions\":[],\"pinned\":false,\"reactions\":[],\"message_reference\":null,\"timestamp\":\"2020-11-20T04:46:34.784000+00:00\",\"tts\":false,\"webhook_id\":null}}";

    let event: Event = serde_json::from_str(&s).unwrap();

    println!("{:#?}", event);

    assert_eq!(event.shard_id, 0);
}

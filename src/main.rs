use darkredis::ConnectionPool;
use std::{env, error::Error};
use tokio::stream::StreamExt;
use tracing_subscriber;
use twilight_gateway::{
    EventTypeFlags,
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_model::gateway::Intents;

#[derive(Clone, Debug)]
struct Context {
    pub redis_pool: ConnectionPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN")?;
    let redis_addr = env::var("REDIS_HOST")?;
    let redis_pool = ConnectionPool::create(redis_addr, None, num_cpus::get()).await?;

    let context = Context { redis_pool };

    let scheme = ShardScheme::Auto;
    let cluster = Cluster::builder(&token, Intents::GUILD_MESSAGES)
        .shard_scheme(scheme)
        .build()
        .await?;

    let cluster_spawn = cluster.clone();

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // Filter only select events
    let types = EventTypeFlags::MESSAGE_CREATE;
    let mut events = cluster.some_events(types);

    while let Some((shard_id, event)) = events.next().await {
        tokio::spawn(handle_event(shard_id, event, context.clone()));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    ctx: Context,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => {
            tracing::info!("Recieved message: {}", msg.0.content);
            let event_str = serde_json::to_string(&msg)?;

            let mut conn = ctx.redis_pool.get().await;
            conn.rpush("events", event_str).await?;
            conn.ltrim("events", 0, 99).await?;
        }
        Event::ShardConnected(_) => {
            tracing::info!("Connected on shard {}", shard_id);
        }
        _ => {}
    }


    Ok(())
}

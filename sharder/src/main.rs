use darkredis::ConnectionPool;
use std::{env, error::Error};
use tokio::signal::unix::{signal, SignalKind};
use tokio::stream::StreamExt;
use tracing_subscriber;
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event, EventTypeFlags,
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

    let signal_kinds = vec![
        SignalKind::hangup(),
        SignalKind::interrupt(),
        SignalKind::terminate(),
    ];

    // Listen for shutdown signals
    for signal_kind in signal_kinds {
        let mut stream = signal(signal_kind).unwrap();
        let cluster = cluster.clone();

        tokio::spawn(async move {
            stream.recv().await;
            tracing::info!("Signal received, shutting down...");
            cluster.down();

            tracing::info!("bye");
        });
    }

    // Filter only select events
    let types = EventTypeFlags::READY | EventTypeFlags::RESUMED | EventTypeFlags::SHARD_PAYLOAD;

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
    match &event {
        Event::MessageCreate(msg) => {
            tracing::info!(
                "Received message: {}#{}: {}",
                msg.0.author.name,
                msg.0.author.discriminator,
                msg.0.content
            );
        }
        Event::Ready(ready) => {
            tracing::info!(
                "Shard {}, user {} ready. {} guilds connected",
                shard_id,
                format!("{}#{}", ready.user.name, ready.user.discriminator),
                ready.guilds.len()
            );
        }
        Event::Resumed => {
            tracing::info!("Resuming shard {}", shard_id);
        }
        Event::ShardPayload(payload) => {
            let mut conn = ctx.redis_pool.get().await;
            conn.rpush("events", &payload.bytes).await?;
        }
        _ => {}
    }

    Ok(())
}

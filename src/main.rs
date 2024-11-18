mod app;
mod api;

use dotenv::dotenv;
use std::{env};
use poise::serenity_prelude as serenity;
use tracing_subscriber;
use crate::app::{
    utils::{database::Database, cache::Cache},
    commands::review,
    events,
};
use tracing::{info, error, Level};
use tokio::task;
use crate::api::server::create_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_ansi(true)
        .with_max_level(Level::INFO)
        .with_env_filter(
            tracing_subscriber::EnvFilter::new("info")
                .add_directive("serenity=error".parse().unwrap())
                .add_directive("tracing=error".parse().unwrap())
        )
        .init();

    info!("Initializing application");
    let token = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");
    let database_url = env::var("DATABASE_URL").expect("Missing DATABASE_URL");
    let redis_url = env::var("REDIS_URL").expect("Missing REDIS_URL");
    let api_port = env::var("API_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("Invalid API_PORT");

    info!("Initializing cache");
    let cache = Cache::new(&redis_url)?;
    let api_cache = cache.clone();

    info!("Establishing database connection");
    let db = Database::new(&database_url, cache).await?;
    let api_db = db.clone();

    let api_task = task::spawn(async move {
        info!("Starting API server on port {}", api_port);
        let app = create_server(api_db, api_cache).await;

        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], api_port));
        if let Err(e) = axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await {
                error!("API server error: {}", e);
        }
    });

    info!("Starting Discord bot");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![review::review()],
            event_handler: |ctx, event, _framework, data| {
                Box::pin(async move {
                    match event {
                        poise::Event::Ready { data_about_bot } => {
                            events::ready(&ctx, data_about_bot).await;
                        }
                        poise::Event::InteractionCreate { interaction } => {
                            events::interaction_create(ctx.clone(), interaction.clone(), &data.db).await;
                        }
                        _ => {}
                    }
                    Ok(())
                })
            },
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(review::Data { db: db.clone() })
            })
        })
        .build()
        .await?;

    tokio::select! {
        _ = framework.start() => {
            error!("Discord bot stopped unexpectedly");
        }
        _ = api_task => {
            error!("API server stopped unexpectedly");
        }
    }

    info!("Application shutting down");
    Ok(())
}
use std::sync::Arc;

use axum::{extract::FromRef, routing::post, Router};
use seatalk_tgs::{
    config::AppConfig,
    seatalk_api::{auth::Auth, seatalk::AsyncSeatalk},
    telegram::TelegramStickerDownloader,
    webhook::message_received,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
struct AppState {
    telegram: Arc<TelegramStickerDownloader>,
    seatalk: Arc<AsyncSeatalk>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let config = AppConfig::new().expect("Failed to parse config");
    let telegram = Arc::new(
        TelegramStickerDownloader::new(&config.telegram.api_token)
            .await
            .expect("Failed to create Telegram client"),
    );
    let seatalk = Arc::new(
        AsyncSeatalk::new(
            "https",
            &config.seatalk.host,
            Auth {
                app_id: config.seatalk.app_id,
                app_secret: config.seatalk.app_secret,
            },
        )
        .await
        .expect("Failed to create seatalk client"),
    );
    let state = AppState { telegram, seatalk };

    let router = Router::new()
        .route("/", post(message_received))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router)
        .await
        .expect("Failed to start server");
}

impl FromRef<AppState> for Arc<TelegramStickerDownloader> {
    fn from_ref(input: &AppState) -> Self {
        input.telegram.clone()
    }
}

impl FromRef<AppState> for Arc<AsyncSeatalk> {
    fn from_ref(input: &AppState) -> Self {
        input.seatalk.clone()
    }
}

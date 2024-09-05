use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use base64::{engine::general_purpose, Engine};
use http::StatusCode;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use temp_dir::TempDir;
use thiserror::Error;

use crate::{
    consts::{GROUP_INV, WHITELIST_GROUP_IDS},
    convert::{convert_tgs, convert_webm, convert_webp},
    seatalk_api::{
        api::{common::MessageType, ApiError, SendGroupMessage, SendSubscriberMessage},
        ignore,
        query::AsyncQuery,
        seatalk::{AsyncSeatalk, RestError, SeatalkError},
        webhooks::{
            MentionedFromGroupChatEvent, MentionedMessage, MentionedMessageContent,
            ReceivedMessage, SeatalkChallengeEvent, SubscriberMessageEvent,
        },
    },
    telegram::TelegramStickerDownloader,
};

pub async fn message_received(
    State(seatalk): State<Arc<AsyncSeatalk>>,
    State(telegram): State<Arc<TelegramStickerDownloader>>,
    Json(payload): Json<ReceivedMessage>,
) -> Result<impl IntoResponse, WebhookError> {
    match payload {
        ReceivedMessage::EventVerification {
            event: SeatalkChallengeEvent { seatalk_challenge },
            ..
        } => {
            return Ok((
                StatusCode::OK,
                Json(json!({"seatalk_challenge": seatalk_challenge})),
            )
                .into_response())
        }
        ReceivedMessage::MessageFromBotSubscriber {
            event: SubscriberMessageEvent { employee_code, .. },
            ..
        } => {
            let seatalk = seatalk.as_ref();
            ignore(SendSubscriberMessage::new(
                &employee_code,
                MessageType::Text,
                "Join my group to convert Telegram stickers!",
            ))
            .query_async(seatalk)
            .await?;
            ignore(SendSubscriberMessage::new(
                &employee_code,
                MessageType::Image,
                GROUP_INV,
            ))
            .query_async(seatalk)
            .await?;
        }
        ReceivedMessage::NewMentionedMessageFromGroupChat {
            event:
                MentionedFromGroupChatEvent {
                    group_id,
                    message:
                        MentionedMessage {
                            message_id,
                            thread_id,
                            text: MentionedMessageContent { plain_text, .. },
                            ..
                        },
                },
            ..
        } => {
            if WHITELIST_GROUP_IDS.contains(&group_id.as_str()) {
                let thread_id = thread_id.unwrap_or("".into());
                if thread_id.is_empty() {
                    let thread_id = Some(message_id);
                    tokio::spawn(async move {
                        let _ = download_and_send_stickers_group(
                            telegram,
                            seatalk.clone(),
                            group_id,
                            thread_id,
                            plain_text,
                        )
                        .await;
                    });
                }
            } else {
                let seatalk = seatalk.as_ref();
                ignore(SendGroupMessage::new(
                    &group_id,
                    None,
                    "Join my group to convert Telegram stickers!",
                    MessageType::Text,
                    None,
                ))
                .query_async(seatalk)
                .await
                .map_err(WebhookError::Rest)?;
                ignore(SendGroupMessage::new(
                    &group_id,
                    None,
                    GROUP_INV,
                    MessageType::Image,
                    None,
                ))
                .query_async(seatalk)
                .await
                .map_err(WebhookError::Rest)?;
            }
        }
    };
    Ok(StatusCode::OK.into_response())
}

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error(transparent)]
    FS(#[from] std::io::Error),

    #[error(transparent)]
    Telegram(#[from] teloxide::RequestError),

    #[error(transparent)]
    Seatalk(#[from] SeatalkError),

    #[error(transparent)]
    Rest(#[from] ApiError<RestError>),

    #[error("Cannot parse telegram url: {0}")]
    BadRequest(String),
}

impl IntoResponse for WebhookError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Error handling request: {:?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(())).into_response()
    }
}

fn parse_sticker_set_name(message: &str) -> Result<String, WebhookError> {
    let re = Regex::new(r"(?:@.* *)(?:/convert *)https://t\.me/(?:addstickers|addemoji)/(.*)/?$")
        .unwrap();
    let caps = re
        .captures(message)
        .ok_or(WebhookError::BadRequest("Cannot capture URL".into()))?;
    let set_name = caps
        .get(1)
        .ok_or(WebhookError::BadRequest("Cannot capture URL".into()))?
        .as_str();
    tracing::info!("Parsed sticker pack: {}", set_name);
    Ok(set_name.into())
}

async fn download_and_send_stickers_group(
    telegram: impl AsRef<TelegramStickerDownloader>,
    seatalk: Arc<AsyncSeatalk>,
    group_id: String,
    quoted_message_id: Option<String>,
    message: String,
) -> Result<(), WebhookError> {
    let telegram = telegram.as_ref();
    let seatalk = seatalk.as_ref();

    let Ok(sticker_set_name) = parse_sticker_set_name(&message) else {
        ignore(SendGroupMessage::new(
            &group_id,
            None,
            "Invalid Telegram sticker set URL\nExample usage: `@StickersBot /convert  https://t.me/addstickers/Trashhagain`",
            MessageType::Text,
            quoted_message_id,
        ))
        .query_async(seatalk)
        .await
        .map_err(WebhookError::Rest)?;
        return Ok(());
    };

    let Ok(sticker_set) = telegram.get_sticker_set(&sticker_set_name).await else {
        ignore(SendGroupMessage::new(
            &group_id,
            None,
            "Invalid Telegram sticker set URL",
            MessageType::Text,
            quoted_message_id,
        ))
        .query_async(seatalk)
        .await
        .map_err(WebhookError::Rest)?;
        return Ok(());
    };

    #[derive(Debug, Deserialize)]
    struct SentGroupMessage {
        message_id: String,
    }

    let code: SentGroupMessage = SendGroupMessage::new(
        &group_id,
        None,
        format!(
            "Found {} stickers in sticker set: **{}**",
            sticker_set.stickers.len(),
            sticker_set.name,
        ),
        MessageType::Text,
        quoted_message_id,
    )
    .query_async(seatalk)
    .await
    .map_err(WebhookError::Rest)?;

    let thread_id = Some(code.message_id);

    let temp_dir = TempDir::new().map_err(WebhookError::FS)?;
    let converted_dir = temp_dir.path().join("converted");
    tokio::fs::create_dir_all(&converted_dir).await?;
    let mut failed = 0;

    for (i, sticker) in sticker_set.stickers.iter().enumerate() {
        tracing::info!(
            "Processing {}: {}/{}",
            sticker_set.name,
            i + 1,
            sticker_set.stickers.len()
        );
        let file_name = sticker.file.id.to_owned();
        let file_path = temp_dir.path().join(&file_name);

        if telegram
            .download_sticker(sticker, &file_path)
            .await
            .is_err()
        {
            tracing::error!("Failed to download sticker");
            failed += 1;
            continue;
        }

        let mut converted_file_path = converted_dir.join(&file_name);
        let cnv = if sticker.flags.is_video {
            converted_file_path.set_extension("gif");
            convert_webm(&file_path, &converted_file_path)
        } else if sticker.flags.is_animated {
            converted_file_path.set_extension("gif");
            convert_tgs(&file_path, &converted_file_path)
        } else {
            converted_file_path.set_extension("png");
            convert_webp(&file_path, &converted_file_path)
        };

        if let Err(e) = cnv {
            tracing::error!("Failed to convert: {}", e);
            failed += 1;
            continue;
        };

        let f = tokio::fs::read(&converted_file_path).await.unwrap();
        let f_b64 = general_purpose::STANDARD.encode(f);
        if ignore(SendGroupMessage::new(
            &group_id,
            thread_id.clone(),
            f_b64,
            MessageType::Image,
            None,
        ))
        .query_async(seatalk)
        .await
        .is_err()
        {
            tracing::error!("Failed to send converted");
            failed += 1;
        }
    }
    if failed == 0 {
        ignore(SendGroupMessage::new(
            &group_id,
            thread_id.clone(),
            "Done",
            MessageType::Text,
            None,
        ))
        .query_async(seatalk)
        .await
        .map_err(WebhookError::Rest)?;
    } else {
        ignore(SendGroupMessage::new(
            &group_id,
            thread_id.clone(),
            format!(
                "Converted {} of {} stickers.\nSome sticker types are not supported yet ):",
                sticker_set.stickers.len() - failed,
                sticker_set.stickers.len()
            ),
            MessageType::Text,
            None,
        ))
        .query_async(seatalk)
        .await
        .map_err(WebhookError::Rest)?;
    }

    let _ = temp_dir.cleanup();
    Ok(())
}

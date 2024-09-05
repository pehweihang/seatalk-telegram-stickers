use backon::{ExponentialBuilder, Retryable};
use std::path::Path;

use teloxide::{
    net::Download,
    requests::Requester,
    types::{Sticker, StickerSet},
    Bot,
};
use tokio::fs;

#[derive(Debug)]
pub struct TelegramStickerDownloader {
    bot: Bot,
}

impl TelegramStickerDownloader {
    pub async fn new(api_token: &str) -> Result<Self, teloxide::RequestError> {
        let bot = Bot::new(api_token);
        let _ = bot.get_me().await?;
        Ok(Self { bot })
    }

    pub async fn get_sticker_set(&self, name: &str) -> Result<StickerSet, teloxide::RequestError> {
        self.bot.get_sticker_set(name).await
    }

    pub async fn download_sticker(
        &self,
        sticker: &Sticker,
        path: impl AsRef<Path>,
    ) -> Result<(), teloxide::RequestError> {
        let file = self.bot.get_file(&sticker.file.id).await?;
        let mut dest = fs::File::create(path).await?;
        self.bot.download_file(&file.path, &mut dest).await?;
        Ok(())
    }

    pub async fn download_sticker_retry(
        &self,
        sticker: &Sticker,
        path: impl AsRef<Path>,
    ) -> Result<(), teloxide::RequestError> {
        let f = || async { self.download_sticker(sticker, &path).await };
        f.retry(ExponentialBuilder::default()).await?;
        Ok(())
    }
}

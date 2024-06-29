//! # core_telegram
//! Набор утилит для работы с api telegram

use teloxide::prelude::*;
use teloxide::payloads::GetUpdates;
use teloxide::requests::JsonRequest;

/// Набор расширений для [Bot].
pub trait TgBotCommands {
    /// Получает список обновлений начиная с [event_id].
    /// Обратите внимание, если ранее бот передавал больший [event_id] то телега это запомнит
    /// и вернет только более новые события.
    /// Можно передать 0 в качестве [event_id] что бы получить максимум возможных событий.
    #[allow(async_fn_in_trait)]
    async fn get_updates_start_from(self, event_id: i32) -> Vec<Update>;
}

impl TgBotCommands for Bot {
    async fn get_updates_start_from(self, event_id: i32) -> Vec<Update> {
        let get_updates = GetUpdates {
            offset: Some(event_id),
            ..Default::default()
        };
        JsonRequest::new(self, get_updates).await.unwrap()
    }
}
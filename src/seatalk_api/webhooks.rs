use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "event_type")]
pub enum ReceivedMessage {
    #[serde(alias = "event_verification")]
    EventVerification {
        event_id: String,
        timestamp: u64,
        app_id: String,
        event: SeatalkChallengeEvent,
    },
    #[serde(alias = "message_from_bot_subscriber")]
    MessageFromBotSubscriber {
        event_id: String,
        timestamp: u64,
        app_id: String,
        event: SubscriberMessageEvent,
    },
    #[serde(alias = "new_mentioned_message_received_from_group_chat")]
    NewMentionedMessageFromGroupChat {
        event_id: String,
        timestamp: u64,
        app_id: String,
        event: MentionedFromGroupChatEvent,
    },
}

#[derive(Debug, Deserialize)]
pub struct SeatalkChallengeEvent {
    pub seatalk_challenge: String,
}

#[derive(Debug, Deserialize)]
pub struct SubscriberMessageEvent {
    pub employee_code: String,
    pub message: SubscriberMessage,
}

#[derive(Debug, Deserialize)]
pub struct SubscriberMessage {
    pub tag: String,
    pub text: SubscriberMessageContent,
}

#[derive(Debug, Deserialize)]
pub struct SubscriberMessageContent {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct MentionedFromGroupChatEvent {
    pub group_id: String,
    pub message: MentionedMessage,
}

#[derive(Debug, Deserialize)]
pub struct MentionedMessage {
    pub message_id: String,
    pub quoted_message_id: String,
    pub thread_id: Option<String>,
    pub sender: Sender,
    pub message_sent_time: u64,
    pub tag: String,
    pub text: MentionedMessageContent,
}

#[derive(Debug, Deserialize)]
pub struct Sender {
    pub seatalk_id: String,
    pub employee_code: String,
    pub sender_type: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct MentionedMessageContent {
    pub plain_text: String,
    pub mentioned_list: Vec<Mention>,
}
#[derive(Debug, Deserialize)]
pub struct Mention {
    pub username: String,
    pub seatalk_id: String,
}

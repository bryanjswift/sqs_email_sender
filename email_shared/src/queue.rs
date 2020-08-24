use rusoto_sqs::{DeleteMessageBatchRequestEntry, Message};
use serde::Deserialize;
use serde_json;
use std::convert::TryFrom;

#[derive(Deserialize, Debug)]
struct EmailPointer {
    email_id: String,
}

impl EmailPointer {
    fn from_json(json: String) -> Option<EmailPointer> {
        serde_json::from_str(&json).ok()
    }
}

#[derive(Clone, Debug)]
pub struct EmailPointerMessage {
    message_id: String,
    handle: String,
    pub email_id: String,
}

impl EmailPointerMessage {
    pub fn from_message(message: Message) -> Option<EmailPointerMessage> {
        EmailPointerMessage::try_from(message).ok()
    }
}

impl TryFrom<Message> for EmailPointerMessage {
    type Error = &'static str;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        let id = message.message_id;
        let handle = message.receipt_handle;
        let body = message.body.map(EmailPointer::from_json).flatten();
        match (id, handle, body) {
            (Some(id), Some(handle), Some(pointer)) => Ok(EmailPointerMessage {
                message_id: id,
                handle: handle,
                email_id: pointer.email_id,
            }),
            (None, _, _) => Err("No message id was found"),
            (Some(_), None, _) => Err("No receipt handle for message"),
            _ => Err("Unable to parse message."),
        }
    }
}

impl std::fmt::Display for EmailPointerMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EmailIdMessage")
            .field("email_id", &self.email_id)
            .field("message_id", &self.message_id)
            .finish()
    }
}

impl From<&EmailPointerMessage> for DeleteMessageBatchRequestEntry {
    fn from(message: &EmailPointerMessage) -> Self {
        DeleteMessageBatchRequestEntry {
            id: message.message_id.clone(),
            receipt_handle: message.handle.clone(),
        }
    }
}

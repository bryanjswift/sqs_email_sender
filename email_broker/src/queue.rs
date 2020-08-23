use rusoto_dynamodb::{AttributeValue, GetItemInput};
use rusoto_sqs::{DeleteMessageBatchRequestEntry, Message};
use serde::Deserialize;
use serde_json;
use std::convert::TryFrom;

#[derive(Deserialize, Debug)]
struct EmailPointer {
    email_id: String,
}

impl EmailPointer {
    fn from_json(json: Option<String>) -> Option<EmailPointer> {
        match json.map(|json| serde_json::from_str(&json)) {
            Some(Ok(pointer)) => Some(pointer),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EmailPointerMessage {
    message_id: String,
    handle: String,
    email_id: String,
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
        let body = EmailPointer::from_json(message.body);
        match (id, handle, body) {
            (Some(id), Some(handle), Some(pointer)) => Ok(EmailPointerMessage {
                message_id: id,
                handle: handle,
                email_id: pointer.email_id,
            }),
            _ => Err("Unable to parse message body."),
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

impl From<&EmailPointerMessage> for GetItemInput {
    fn from(message: &EmailPointerMessage) -> Self {
        let email_id_attribute = AttributeValue {
            s: Some(message.email_id.clone()),
            ..AttributeValue::default()
        };
        let mut input = GetItemInput::default();
        input
            .key
            .insert(String::from("EmailId"), email_id_attribute);
        input
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

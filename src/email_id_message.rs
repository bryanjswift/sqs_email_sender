use rusoto_dynamodb::{AttributeValue, GetItemInput};
use rusoto_sqs::Message;
use serde::Deserialize;
use serde_json;

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
pub struct EmailIdMessage {
    message_id: String,
    handle: String,
    email_id: String,
}

impl EmailIdMessage {
    pub fn from_message(message: Message) -> Option<EmailIdMessage> {
        let id = message.message_id;
        let handle = message.receipt_handle;
        let body = EmailPointer::from_json(message.body);
        match (id, handle, body) {
            (Some(id), Some(handle), Some(pointer)) => Some(EmailIdMessage {
                message_id: id,
                handle: handle,
                email_id: pointer.email_id,
            }),
            _ => None,
        }
    }
}

impl std::fmt::Display for EmailIdMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EmailIdMessage")
            .field("email_id", &self.email_id)
            .field("message_id", &self.message_id)
            .finish()
    }
}

impl From<EmailIdMessage> for GetItemInput {
    fn from(message: EmailIdMessage) -> Self {
        let email_id_attribute = AttributeValue {
            s: Some(message.email_id),
            ..AttributeValue::default()
        };
        let mut input = GetItemInput {
            table_name: String::from("emails_test_db"),
            ..GetItemInput::default()
        };
        input
            .key
            .insert(String::from("EmailId"), email_id_attribute);
        input
    }
}

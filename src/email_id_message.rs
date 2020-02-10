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

#[derive(Debug)]
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

    pub fn as_dynamodb_input(self) -> GetItemInput {
        let email_id_attribute = AttributeValue {
            s: Some(self.email_id),
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

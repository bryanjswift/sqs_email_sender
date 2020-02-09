use rusoto_core::{Region, RusotoError};
use rusoto_sqs::{Message, ReceiveMessageError, ReceiveMessageRequest, Sqs, SqsClient};
use serde::Deserialize;
use serde_json;

#[tokio::main]
async fn main() {
    let queue = SqsClient::new(Region::UsEast1);
    let messages_result = get_sqs_email_messages(queue.clone()).await;
    match messages_result {
        Ok(Some(messages)) => println!("Process messages, {:?}", messages),
        Ok(None) => println!("No messages"),
        Err(error) => println!("{}", error),
    }
}

async fn get_sqs_email_messages(
    queue: SqsClient,
) -> Result<Option<SqsEmailMessages>, RusotoError<ReceiveMessageError>> {
    let request = ReceiveMessageRequest {
        attribute_names: Some(vec![String::from("MessageGroupId")]),
        max_number_of_messages: Some(1),
        message_attribute_names: None,
        queue_url: String::from(
            "https://sqs.us-east-1.amazonaws.com/161895662097/email_test_queue",
        ),
        receive_request_attempt_id: None,
        visibility_timeout: Some(10),
        wait_time_seconds: Some(20),
    };
    match queue.receive_message(request).await {
        Ok(result) => Ok(result.messages.map(SqsEmailMessages::new)),
        Err(error) => Err(error),
    }
}

#[derive(Debug)]
struct SqsEmailMessages {
    messages: Vec<Message>,
}

impl SqsEmailMessages {
    fn new(messages: Vec<Message>) -> SqsEmailMessages {
        SqsEmailMessages { messages: messages }
    }
}

impl Iterator for SqsEmailMessages {
    type Item = SqsEmailMessage;

    fn next(&mut self) -> Option<SqsEmailMessage> {
        if self.messages.is_empty() {
            return None;
        }
        let message = self.messages.remove(0);
        let email = SqsEmailMessage::from_message(message);
        match email {
            Some(item) => Some(item),
            None => self.next(),
        }
    }
}

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
struct SqsEmailMessage {
    id: String,
    handle: String,
    email_id: String,
}

impl SqsEmailMessage {
    fn from_message(message: Message) -> Option<SqsEmailMessage> {
        let id = message.message_id;
        let handle = message.receipt_handle;
        let body = EmailPointer::from_json(message.body);
        match (id, handle, body) {
            (Some(id), Some(handle), Some(pointer)) => Some(SqsEmailMessage {
                id: id,
                handle: handle,
                email_id: pointer.email_id,
            }),
            _ => None,
        }
    }
}

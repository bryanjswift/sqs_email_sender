mod sqs_email_message;
mod sqs_email_messages;

use log::{info, error};
use rusoto_core::{Region, RusotoError};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient};
use rusoto_sqs::{ReceiveMessageError, ReceiveMessageRequest, Sqs, SqsClient};
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};

use sqs_email_messages::SqsEmailMessages;

#[tokio::main]
async fn main() {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap();
    let queue = SqsClient::new(Region::UsEast1);
    let messages_result = get_sqs_email_messages(queue.clone()).await;
    match messages_result {
        Ok(messages) => process_messages(messages).await,
        Err(error) => error!("{}", error),
    }
}

async fn process_messages(messages: SqsEmailMessages) {
    info!("Process messages, {:?}", messages);
    let client = DynamoDbClient::new(Region::UsEast1);
    for message in messages {
        match client.get_item(message.as_dynamodb_input()).await {
            Ok(item) => info!("{:?}", item),
            Err(error) => error!("{}", error),
        }
    }
}

async fn get_sqs_email_messages(
    queue: SqsClient,
) -> Result<SqsEmailMessages, RusotoError<ReceiveMessageError>> {
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
        Ok(result) => Ok(SqsEmailMessages::new(result.messages.unwrap_or(Vec::new()))),
        Err(error) => Err(error),
    }
}

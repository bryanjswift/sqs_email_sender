mod email_id_message;
mod sqs_email_messages;

use log::{error, info};
use rusoto_core::{Region, RusotoError};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, GetItemInput};
use rusoto_sqs::{ReceiveMessageError, ReceiveMessageRequest, Sqs, SqsClient};
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};

use email_id_message::EmailIdMessage;
use sqs_email_messages::SqsEmailMessages;

#[derive(Clone, Debug)]
struct Config {
    queue_url: String,
    region: Region,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            queue_url: String::default(),
            region: Region::UsEast1,
        }
    }
}

#[tokio::main]
async fn main() {
    TermLogger::init(LevelFilter::Info, LogConfig::default(), TerminalMode::Mixed).unwrap();
    let config = Config {
        queue_url: std::env::var("QUEUE_URL").unwrap_or(String::from(
            "https://sqs.us-east-1.amazonaws.com/161895662097/email_test_queue",
        )),
        ..Config::default()
    };
    let queue = SqsClient::new(Region::UsEast1);
    let messages_result = get_sqs_email_messages(config, queue).await;
    match messages_result {
        Ok(messages) => process_messages(messages).await,
        Err(error) => error!("{}", error),
    }
}

async fn process_messages(messages: SqsEmailMessages) {
    info!("Process messages, {:?}", messages);
    let client = DynamoDbClient::new(Region::UsEast1);
    for message in messages {
        match client.get_item(GetItemInput::from(message)).await {
            Ok(item) => info!("{:?}", item),
            Err(error) => error!("{}", error),
        }
    }
}

async fn get_sqs_email_messages(
    config: Config,
    queue: SqsClient,
) -> Result<SqsEmailMessages, RusotoError<ReceiveMessageError>> {
    let request = ReceiveMessageRequest {
        attribute_names: Some(vec![String::from("MessageGroupId")]),
        max_number_of_messages: Some(1),
        message_attribute_names: None,
        queue_url: config.queue_url,
        receive_request_attempt_id: None,
        visibility_timeout: Some(10),
        wait_time_seconds: Some(20),
    };
    match queue.receive_message(request).await {
        Ok(result) => Ok(SqsEmailMessages::new(result.messages.unwrap_or(Vec::new()))),
        Err(error) => Err(error),
    }
}

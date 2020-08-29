mod config;

use log::{error, info};
use rusoto_core::RusotoError;
use rusoto_dynamodb::DynamoDbClient;
use rusoto_sqs::{
    DeleteMessageBatchRequest, DeleteMessageBatchRequestEntry, Message, ReceiveMessageError,
    ReceiveMessageRequest, Sqs, SqsClient,
};
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use structopt::StructOpt;

use config::Options;
use email_shared::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    TermLogger::init(LevelFilter::Info, LogConfig::default(), TerminalMode::Mixed).unwrap();
    let opt = Options::from_args();
    info!("{:?}", opt);
    let sqs = SqsClient::new(opt.region.clone());
    let dynamodb = DynamoDbClient::new(opt.region.clone());
    let client = Client::new(&dynamodb, &opt.table_name);
    let queue_url = &opt.queue_url;
    loop {
        let message_list = get_sqs_email_messages(queue_url, &sqs).await;
        let processed_messages = match message_list {
            Ok(messages) => client.process_messages(messages).await,
            Err(error) => {
                error!("get_sqs_email_messages: {}", error);
                Vec::new()
            }
        };
        let entries_to_delete = processed_messages
            .iter()
            .map(DeleteMessageBatchRequestEntry::from)
            .collect();
        let delete_messages_request = DeleteMessageBatchRequest {
            entries: entries_to_delete,
            queue_url: queue_url.into(),
        };
        info!("{:?}", delete_messages_request);
        if opt.dry_run {
            break;
        }
    }
    Ok(())
}

/// Poll SQS at the given `queue_url` for new messages providing an iterator for `EmailIdMessage`.
async fn get_sqs_email_messages(
    queue_url: &str,
    sqs: &SqsClient,
) -> Result<Vec<Message>, RusotoError<ReceiveMessageError>> {
    let request = ReceiveMessageRequest {
        attribute_names: Some(vec![String::from("MessageGroupId")]),
        max_number_of_messages: Some(1),
        queue_url: queue_url.into(),
        visibility_timeout: Some(30),
        wait_time_seconds: Some(20),
        ..ReceiveMessageRequest::default()
    };
    sqs.receive_message(request)
        .await
        .map(|result| result.messages.unwrap_or(Vec::new()))
}

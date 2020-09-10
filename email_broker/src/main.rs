mod config;

use rusoto_core::RusotoError;
use rusoto_dynamodb::DynamoDbClient;
use rusoto_sqs::{
    DeleteMessageBatchRequest, DeleteMessageBatchRequestEntry, Message, ReceiveMessageError,
    ReceiveMessageRequest, Sqs, SqsClient,
};
use structopt::StructOpt;
use tracing::{event, span, Level};

use config::Options;
use email_shared::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup Logger
    let subscriber = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc3339())
        .finish();
    let _subscriber_guard = tracing::subscriber::set_global_default(subscriber);
    let main_span = span!(
        Level::INFO,
        "email_broker",
        Version = env!("CARGO_PKG_VERSION"),
    );
    let _main_guard = main_span.enter();
    // Start
    let opt = Options::from_args();
    event!(
        Level::INFO,
        queue_url = %opt.queue_url,
        region = %opt.region.name(),
        table_name = %opt.table_name,
        "broker init",
    );
    let sqs = SqsClient::new(opt.region.clone());
    let dynamodb = DynamoDbClient::new(opt.region.clone());
    let client = Client::new(&dynamodb, &opt.table_name);
    let queue_url = &opt.queue_url;
    loop {
        let message_list = get_sqs_email_messages(queue_url, &sqs).await;
        let processed_messages = match message_list {
            Ok(messages) => client.process_messages(messages).await,
            Err(error) => {
                event!(Level::ERROR, %error, "ReceiveMessageError");
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
        event!(
            Level::INFO,
            count = delete_messages_request.entries.len(),
            "delete messages"
        );
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

mod attribute_value_wrapper;
mod email_id_message;
mod email_message;
mod sqs_email_messages;

use log::{error, info};
use rusoto_core::{Region, RusotoError};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, GetItemInput};
use rusoto_sqs::{DeleteMessageBatchRequest, DeleteMessageBatchRequestEntry};
use rusoto_sqs::{ReceiveMessageError, ReceiveMessageRequest, Sqs, SqsClient};
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use std::convert::TryFrom;
use std::env;

use email_id_message::EmailIdMessage;
use email_message::{EmailMessage, ParseEmailMessageCode};
use sqs_email_messages::SqsEmailMessages;

/// Hold references to external service clients so they only need to be allocated once.
struct Client<'a> {
    /// Connection to DynamoDB
    dynamodb: &'a DynamoDbClient,
    /// Connection to SQS
    sqs: &'a SqsClient,
}

/// Defines the configuration for how the email service executable will interact with external
/// services.
#[derive(Clone, Debug, Default)]
struct Config {
    /// Whether or not this run is a dry run.
    dry_run: bool,
    /// From which email message ids will be read.
    queue_url: String,
    /// Region from which services provided by AWS will be accessed.
    region: Region,
    /// DynamoDB table from which email data will be read.
    table_name: String,
}

impl Config {
    /// Creates a default `Config` by reading the environment variables.
    ///
    /// * `AWS_REGION` corresponds to the region contacted when accessing the services provided by
    /// AWS. Defaults to a `Region::Custom` for
    /// [LocalStack](https://github.com/localstack/localstack) if the environment variable does not
    /// exist.
    /// * `DRY_RUN` indicates emails should not be transmitted and messages should not be deleted
    /// from the queue.
    /// * `QUEUE_URL` defines the queue from which messages will be read.
    ///
    /// # Panics
    ///
    /// * If a region is provided via the `AWS_REGION` environment variable but fails to be parsed.
    /// * If a `QUEUE_URL` environment variable is not set.
    ///
    fn env() -> Self {
        let dry_run = std::env::var("DRY_RUN")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);
        let region = std::env::var("AWS_REGION")
            .map(|s| match s.parse::<Region>() {
                Ok(region) => region,
                Err(error) => panic!("Unable to parse AWS_REGION={}. {}", s, error),
            })
            .unwrap_or(Region::Custom {
                name: "localstack".into(),
                endpoint: "localhost".into(),
            });
        let queue_url = match std::env::var("QUEUE_URL") {
            Ok(url) => url,
            Err(env::VarError::NotPresent) => panic!("QUEUE_URL must be provided."),
            Err(_) => panic!("QUEUE_URL could not be read."),
        };
        let table_name = match std::env::var("TABLE_NAME") {
            Ok(name) => name,
            Err(env::VarError::NotPresent) => panic!("TABLE_NAME must be provided."),
            Err(_) => panic!("TABLE_NAME could not be read."),
        };
        Config {
            dry_run,
            queue_url,
            region,
            table_name,
            ..Config::default()
        }
    }
}

#[tokio::main]
async fn main() {
    TermLogger::init(LevelFilter::Info, LogConfig::default(), TerminalMode::Mixed).unwrap();
    let config = Config::env();
    info!("{:?}", config);
    let sqs = SqsClient::new(config.region.clone());
    let dynamodb = DynamoDbClient::new(config.region.clone());
    let client = Client {
        dynamodb: &dynamodb,
        sqs: &sqs,
    };
    loop {
        let message_list = get_sqs_email_messages(&config.queue_url, client.sqs).await;
        let processed_messages = match message_list {
            Ok(messages) => process_messages(client.dynamodb, messages).await,
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
            queue_url: config.queue_url.clone(),
        };
        info!("{:?}", delete_messages_request);
        if config.dry_run {
            break;
        }
    }
}

async fn process_messages(
    dynamodb: &DynamoDbClient,
    messages: SqsEmailMessages,
) -> Vec<EmailIdMessage> {
    info!("Process messages, {:?}", messages);
    let mut sent_message_handles = Vec::new();
    for message in messages {
        match process_message(dynamodb, message).await {
            Ok(id_message) => sent_message_handles.push(id_message),
            Err(_) => (),
        }
    }
    sent_message_handles
}

async fn process_message(
    dynamodb: &DynamoDbClient,
    message: EmailIdMessage,
) -> Result<EmailIdMessage, String> {
    let id_message = message.clone();
    let email_message = get_email_message(dynamodb, message).await;
    let send_result = match email_message {
        Ok(email) => send_email(email).await,
        Err(error) => {
            error!("process_message: {}: {}", &id_message, error);
            Err("Unable to Parse Email".into())
        }
    };
    match send_result {
        Ok(_) => Ok(id_message),
        Err(msg) => Err(msg),
    }
}

async fn get_email_message(
    client: &DynamoDbClient,
    message: EmailIdMessage,
) -> Result<EmailMessage, ParseEmailMessageCode> {
    let mut input = GetItemInput::from(message);
    input.table_name = "emails_test_db".into();
    let response = client.get_item(input).await;
    match response {
        Ok(output) => EmailMessage::try_from(output),
        Err(error) => {
            error!("get_email_message: {}", error);
            Err(ParseEmailMessageCode::RecordUnreachable)
        }
    }
}

async fn get_sqs_email_messages(
    queue_url: &str,
    sqs: &SqsClient,
) -> Result<SqsEmailMessages, RusotoError<ReceiveMessageError>> {
    let request = ReceiveMessageRequest {
        attribute_names: Some(vec![String::from("MessageGroupId")]),
        max_number_of_messages: Some(1),
        queue_url: queue_url.into(),
        visibility_timeout: Some(30),
        wait_time_seconds: Some(20),
        ..ReceiveMessageRequest::default()
    };
    match sqs.receive_message(request).await {
        Ok(result) => Ok(SqsEmailMessages::new(result.messages.unwrap_or(Vec::new()))),
        Err(error) => Err(error),
    }
}

async fn send_email(email: EmailMessage) -> Result<(), String> {
    info!("send_email: {:?}", email);
    Err("Unimplemented".into())
}

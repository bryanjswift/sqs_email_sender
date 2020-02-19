mod attribute_value_wrapper;
mod email_id_message;
mod email_message;
mod sqs_email_messages;

use log::{error, info};
use rusoto_core::{Region, RusotoError};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, GetItemInput};
use rusoto_sqs::{ReceiveMessageError, ReceiveMessageRequest, Sqs, SqsClient};
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use std::convert::TryFrom;

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
    /// From which email message ids will be read.
    queue_url: String,
    /// Region from which services provided by AWS will be accessed.
    region: Region,
}

impl Config {
    /// Creates a default `Config` by reading the environment variables.
    ///
    /// * `AWS_REGION` corresponds to the region contacted when accessing the services provided by
    /// AWS. Defaults to a `Region::Custom` for
    /// [LocalStack](https://github.com/localstack/localstack) if the environment variable does not
    /// exist.
    /// * `QUEUE_URL` defines the queue from which messages will be read.
    ///
    /// # Panics
    ///
    /// * If a region is provided via the `AWS_REGION` environment variable but fails to be parsed.
    /// * If a `QUEUE_URL` environment variable is not set.
    ///
    fn env() -> Self {
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
            Err(_) => panic!("QUEUE_URL must be provided."),
        };
        Config {
            queue_url,
            region,
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
    match get_sqs_email_messages(&config.queue_url, client.sqs).await {
        Ok(messages) => process_messages(&client, messages).await,
        Err(error) => error!("get_sqs_email_messages: {}", error),
    }
}

async fn process_messages(client: &Client<'_>, messages: SqsEmailMessages) {
    info!("Process messages, {:?}", messages);
    for message in messages {
        let email_message = get_email_message(client.dynamodb, message).await;
        match email_message {
            Ok(item) => info!("get_email_message: {:?}", item),
            Err(error) => error!("get_email_message: {}", error),
        }
    }
}

async fn get_email_message(
    client: &DynamoDbClient,
    message: EmailIdMessage,
) -> Result<EmailMessage, ParseEmailMessageCode> {
    let response = client.get_item(GetItemInput::from(message)).await;
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
        ..ReceiveMessageRequest::default()
    };
    match sqs.receive_message(request).await {
        Ok(result) => Ok(SqsEmailMessages::new(result.messages.unwrap_or(Vec::new()))),
        Err(error) => Err(error),
    }
}

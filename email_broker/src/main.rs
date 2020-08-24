mod config;

use log::{error, info};
use rusoto_core::RusotoError;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, GetItemInput};
use rusoto_sqs::{DeleteMessageBatchRequest, DeleteMessageBatchRequestEntry, Message};
use rusoto_sqs::{ReceiveMessageError, ReceiveMessageRequest, Sqs, SqsClient};
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use std::convert::TryFrom;
use structopt::StructOpt;

use config::Options;
use email_shared::email_message::{EmailMessage, ParseEmailMessageCode};
use email_shared::queue::EmailPointerMessage;

/// Hold references to external service clients so they only need to be allocated once.
struct Client<'a> {
    /// Connection to DynamoDB
    dynamodb: &'a DynamoDbClient,
    /// Connection to SQS
    sqs: &'a SqsClient,
    /// DynamoDB table from which email data will be read.
    table_name: &'a str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    TermLogger::init(LevelFilter::Info, LogConfig::default(), TerminalMode::Mixed).unwrap();
    let opt = Options::from_args();
    info!("{:?}", opt);
    let sqs = SqsClient::new(opt.region.clone());
    let dynamodb = DynamoDbClient::new(opt.region.clone());
    let client = Client {
        dynamodb: &dynamodb,
        sqs: &sqs,
        table_name: &opt.table_name,
    };
    let queue_url = &opt.queue_url;
    loop {
        let message_list = get_sqs_email_messages(queue_url, client.sqs).await;
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

impl Client<'_> {
    async fn process_messages(&self, messages: Vec<Message>) -> Vec<EmailPointerMessage> {
        info!("Process messages, {:?}", messages);
        let mut processed_message_handles = Vec::new();
        for message in messages {
            match self.process_message(message).await {
                Ok(id_message) => processed_message_handles.push(id_message),
                Err(_) => (), // TODO: This needs to at least log the error
            }
        }
        processed_message_handles
    }

    async fn process_message(&self, message: Message) -> Result<EmailPointerMessage, String> {
        let pointer = EmailPointerMessage::try_from(message)?;
        let email_message = self.get_email_message(&pointer).await;
        let send_result = match email_message {
            Ok(email) => send_email(email).await,
            Err(error) => {
                error!("process_message: {}: {}", &pointer, error);
                Err("Unable to Parse Email".into())
            }
        };
        match send_result {
            Ok(_) => Ok(pointer),
            Err(msg) => Err(msg),
        }
    }

    async fn get_email_message(
        &self,
        message: &EmailPointerMessage,
    ) -> Result<EmailMessage, ParseEmailMessageCode> {
        let mut input = GetItemInput::from(message);
        input.table_name = self.table_name.into();
        self.dynamodb
            .get_item(input)
            .await
            .map_err(ParseEmailMessageCode::from)
            .and_then(EmailMessage::try_from)
    }
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

async fn send_email(email: EmailMessage) -> Result<(), String> {
    info!("send_email: {:?}", email);
    Err("Unimplemented".into())
}

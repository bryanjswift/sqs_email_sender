mod de;
mod error;

use de::MessageDef;
use email_shared::{get_email_message, queue::EmailPointerMessage};
use lambda_runtime::{error::HandlerError, lambda, Context};
use log::info;
use rusoto_core::Region;
use rusoto_dynamodb::DynamoDbClient;
use rusoto_sqs::{DeleteMessageBatchRequest, DeleteMessageBatchRequestEntry, Message};
use serde::{Deserialize, Serialize};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use std::convert::TryFrom;
use std::env;

const DYNAMO_TABLE: &str = "DYNAMO_TABLE";
const QUEUE_URL: &str = "QUEUE_URL";

#[derive(Deserialize, Clone)]
struct SqsEvent {
    #[serde(rename = "Records")]
    records: Vec<MessageDef>,
}

#[derive(Serialize, Clone)]
struct CustomOutput {
    message: String,
}

fn main() {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();
    lambda!(handler);
}

fn handler(event: SqsEvent, _ctx: Context) -> Result<CustomOutput, HandlerError> {
    let mut email_ids = Vec::new();
    let mut entries_to_delete = Vec::new();
    // Create the client to access Dynamo DB
    let dynamodb = DynamoDbClient::new(Region::default());
    // Read dynamo db table name from config or environment
    let table_name = env::var(DYNAMO_TABLE)?;
    // Read queue url from config or environment
    let queue_url = env::var(QUEUE_URL)?;
    for record in event.records.into_iter() {
        // Which errors mean try again and which errors mean skip message?
        // 1. Parse email_id from SQS message
        let message: Message = record.into();
        let pointer = EmailPointerMessage::try_from(message)?;
        // 2. Get email data from dynamo db table
        // 3. Parse dynamo data into object for sending
        info!("Looking for {:?} in {:?}", &pointer.email_id, &table_name);
        let _email = get_email_message(&dynamodb, &table_name, &pointer);
        // 4. TODO: Send the message
        // 5. Register the message for deletion
        entries_to_delete.push(DeleteMessageBatchRequestEntry::from(&pointer));
        email_ids.push(pointer.email_id.clone());
    }
    // Read the queue url from config
    let delete_messages_request = DeleteMessageBatchRequest {
        entries: entries_to_delete,
        queue_url,
    };
    // Delete "processed" messages from SQS
    Ok(CustomOutput {
        message: format!(
            "Hello {:?}! Goodbye {:?}",
            email_ids, delete_messages_request
        ),
    })
}

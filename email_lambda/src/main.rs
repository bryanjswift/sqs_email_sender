mod de;

use de::MessageDef;
use email_shared::queue::EmailPointerMessage;
use lambda_runtime::{error::HandlerError, lambda, Context};
use rusoto_sqs::{DeleteMessageBatchRequest, DeleteMessageBatchRequestEntry, Message};
use serde::{Deserialize, Serialize};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use std::convert::TryFrom;

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
    for record in event.records.into_iter() {
        // Which errors mean try again and which errors mean skip message?
        // 1. Parse email_id from SQS message
        let message: Message = record.into();
        let pointer = EmailPointerMessage::try_from(message)?;
        // 2. Read dynamo db table name from config
        // 3. Get email data from dynamo db table
        // 4. Parse dynamo data into object for sending
        // 5. Send the message
        // 6. Register the message for deletion
        entries_to_delete.push(DeleteMessageBatchRequestEntry::from(&pointer));
        email_ids.push(pointer.email_id);
    }
    // Read the queue url from config
    let delete_messages_request = DeleteMessageBatchRequest {
        entries: entries_to_delete,
        queue_url: "".into(),
    };
    // Delete "processed" messages from SQS
    Ok(CustomOutput {
        message: format!(
            "Hello {:?}! Goodbye {:?}",
            email_ids, delete_messages_request
        ),
    })
}

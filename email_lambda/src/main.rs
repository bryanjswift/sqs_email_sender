mod de;
mod error;

#[macro_use]
extern crate lazy_static;

use de::MessageDef;
use email_shared::email_message::EmailStatus;
use email_shared::queue::EmailPointerMessage;
use email_shared::{get_email_message, set_email_status, UpdateError};
use error::EmailHandlerError;
use lambda_runtime::error::HandlerError;
use lambda_runtime::{lambda, Context};
use rusoto_core::Region;
use rusoto_dynamodb::DynamoDbClient;
use rusoto_sqs::{
    DeleteMessageBatchRequest, DeleteMessageBatchRequestEntry, Message, Sqs, SqsClient,
};
use serde::{Deserialize, Serialize};
use slog::{error, info, slog_o, warn, Drain};
use std::convert::TryFrom;
use std::env;

const DYNAMO_TABLE: &str = "DYNAMO_TABLE";
const QUEUE_URL: &str = "QUEUE_URL";

lazy_static! {
    static ref LOGGER: slog::Logger = get_root_logger();
    static ref DYNAMODB: DynamoDbClient = DynamoDbClient::new(Region::UsEast1);
    static ref SQS: SqsClient = SqsClient::new(Region::UsEast1);
}

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
    lambda!(handler);
}

#[tokio::main]
async fn handler(event: SqsEvent, context: Context) -> Result<CustomOutput, HandlerError> {
    let logger = LOGGER.new(slog_o!(
        "RequestId" => context.aws_request_id,
        "Version" => context.function_version,
    ));
    // Start
    let mut entries_to_delete = Vec::new();
    // Read dynamo db table name from config or environment
    let table_name = env::var(DYNAMO_TABLE)?;
    // Read queue url from config or environment
    let queue_url = env::var(QUEUE_URL)?;
    // Get the number of records received
    let record_count = event.records.len();
    // Process each event record
    for record in event.records.into_iter() {
        // Which errors mean try again and which errors mean skip message?
        // 1. Parse email_id from SQS message
        let message: Message = record.into();
        let pointer = EmailPointerMessage::try_from(message);
        let pointer = match pointer {
            Ok(record) => record,
            Err(msg) => {
                error!(logger, "pointer parse failure"; "msg" => msg);
                continue;
            }
        };
        // Create logger for this record
        let record_logger = logger.new(slog_o!("email_id" => pointer.email_id.clone()));
        // 2. Get email data from dynamo db table
        // 3. Parse dynamo data into object for sending
        info!(record_logger, "get email";
            "table_name" => &table_name
        );
        let email = get_email_message(&DYNAMODB, &table_name, &pointer).await;
        // 4. If status of email is not `EmailStatus::Pending` log a warning and skip sending. The
        //    message to remove will automatically be created.
        let email = match email {
            Ok(mail) if mail.status != EmailStatus::Pending => {
                warn!(record_logger, "email not pending";
                    "email_status" => &mail.status.to_string(),
                );
                // See 7.
                // Skipping doesn't work unless the pointer is recorded as an entry to be deleted.
                entries_to_delete.push(DeleteMessageBatchRequestEntry::from(&pointer));
                continue;
            }
            Ok(mail) => mail,
            Err(error) => {
                error!(record_logger, "get email failed"; "error" => error.to_string());
                continue;
            }
        };
        // 5. Update the message status in dynamo so that a second receiver for this message will
        //    not try to send the same email
        let update_result = upate_to_sending(&table_name, &pointer).await;
        if let Err(error) = update_result {
            error!(record_logger, "update email failed"; "error" => error.to_string());
            continue;
        }
        // 6. TODO: Send the message
        info!(record_logger, "start email transmit";
            "email_status" => &email.status.to_string()
        );
        let update_result = upate_to_sent(&table_name, &pointer).await;
        if let Err(error) = update_result {
            error!(record_logger, "update email failed"; "error" => error.to_string());
            continue;
        }
        // 7. Messages are automatically removed from the queue if lambda succeeds. In case of
        //    failure keep track of the successfully processed messages so in the event of partial
        //    (or total) batch failure the successful messages can be deleted but the errored
        //    messages will get redelivered.
        entries_to_delete.push(DeleteMessageBatchRequestEntry::from(&pointer));
    }
    // Read the queue url from config
    let entries_to_delete_count = entries_to_delete.len();
    if record_count == entries_to_delete_count {
        info!(logger, "success";
            "entries_to_delete" => format!("{:?}", &entries_to_delete),
        );
        Ok(CustomOutput {
            message: format!("Goodbye {:?}", &entries_to_delete),
        })
    } else {
        // Delete "processed" messages from SQS
        info!(logger, "partial failure";
            "entries_to_delete" => format!("{:?}", &entries_to_delete),
        );
        let delete_response = &SQS
            .delete_message_batch(DeleteMessageBatchRequest {
                entries: entries_to_delete,
                queue_url,
            })
            .await;
        let error = match delete_response {
            Ok(_) if entries_to_delete_count > 0 => EmailHandlerError::PartialBatchFailure,
            Ok(_) => EmailHandlerError::BatchFailure,
            Err(_) => EmailHandlerError::SqsDeleteFailed,
        };
        Err(HandlerError::new(error))
    }
}

/// Update the `EmailStatus` of the Dynamo record identified by `pointer` to `EmailStatus::Sending` as
/// long as it is currently in the `EmailStatus::Pending` status.
async fn upate_to_sending(
    table_name: &str,
    pointer: &EmailPointerMessage,
) -> Result<(), UpdateError> {
    set_email_status(
        &DYNAMODB,
        &table_name,
        &pointer,
        EmailStatus::Pending,
        EmailStatus::Sending,
    )
    .await
}

/// Update the `EmailStatus` of the Dynamo record identified by `pointer` to `EmailStatus::Sent` as
/// long as it is currently in the `EmailStatus::Sending` status.
async fn upate_to_sent(table_name: &str, pointer: &EmailPointerMessage) -> Result<(), UpdateError> {
    set_email_status(
        &DYNAMODB,
        &table_name,
        &pointer,
        EmailStatus::Sending,
        EmailStatus::Sent,
    )
    .await
}

/// Lambda writes timestamps to CloudWatch logs already, so do not write anything when asked for a
/// timestamp.
fn noop_timestamp(_: &mut dyn std::io::Write) -> std::io::Result<()> {
    Ok(())
}

/// Create the "root" `slog::Logger` to use.
fn get_root_logger() -> slog::Logger {
    // Setup Logger
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    let drain = slog_term::FullFormat::new(decorator)
        .use_custom_timestamp(noop_timestamp)
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(drain, slog_o!())
}

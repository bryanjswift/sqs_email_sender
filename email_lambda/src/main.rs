mod de;
mod error;

#[macro_use]
extern crate lazy_static;

use de::MessageDef;
use email_shared::Client;
use error::EmailHandlerError;
use rusoto_core::Region;
use rusoto_dynamodb::DynamoDbClient;
use rusoto_sqs::{DeleteMessageBatchRequest, Sqs, SqsClient};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{event, span, Level};
use tracing_futures::Instrument;

const DYNAMO_TABLE: &str = "DYNAMO_TABLE";
const QUEUE_URL: &str = "QUEUE_URL";

lazy_static! {
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

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc3339())
        .finish();
    let _guard = tracing::subscriber::set_global_default(subscriber);
    lambda_runtime::run(lambda_runtime::handler_fn(handler)).await?;
    Ok(())
}

async fn handler(
    event: SqsEvent,
    context: lambda_runtime::Context,
) -> Result<CustomOutput, EmailHandlerError> {
    let handler_span = span!(
        Level::INFO,
        env!("CARGO_PKG_NAME"),
        RequestId = %context.request_id,
        ARN = %context.invoked_function_arn,
    );
    let _handler_guard = handler_span.enter();
    // Read dynamo db table name from config or environment
    let table_name = env::var(DYNAMO_TABLE)?;
    // Read queue url from config or environment
    let queue_url = env::var(QUEUE_URL)?;
    // Get the number of records received for comparison later
    let record_count = event.records.len();
    // Create a shared processing client
    let client = Client::new(&DYNAMODB, &table_name);
    // Process each event record
    let entries_to_delete = client
        .process_messages(event.records.into_iter().map(|record| record.into()))
        .in_current_span()
        .await;
    // Compare the number of messages to be deleted with the number received
    let entries_to_delete_count = entries_to_delete.len();
    if record_count == entries_to_delete_count {
        event!(Level::INFO, ?entries_to_delete, "success");
        Ok(CustomOutput {
            message: format!("Goodbye {:?}", &entries_to_delete),
        })
    } else {
        // Delete "processed" messages from SQS
        event!(Level::INFO, ?entries_to_delete, "partial failure");
        let delete_response = &SQS
            .delete_message_batch(DeleteMessageBatchRequest {
                entries: entries_to_delete,
                queue_url,
            })
            .instrument(tracing::info_span!("delete_message_batch"))
            .await;
        let error = match delete_response {
            Ok(_) if entries_to_delete_count > 0 => EmailHandlerError::PartialBatchFailure,
            Ok(_) => EmailHandlerError::BatchFailure,
            Err(_) => EmailHandlerError::SqsDeleteFailed,
        };
        Err(error)
    }
}

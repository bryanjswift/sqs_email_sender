mod de;

use de::MessageDef;
use email_shared::queue::EmailPointerMessage;
use lambda_runtime::{error::HandlerError, lambda, Context};
use log::info;
use serde::{Deserialize, Serialize};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};

#[derive(Deserialize, Clone)]
struct SqsEvent {
    #[serde(rename = "Records")]
    records: Vec<MessageDef>,
}

#[derive(Deserialize, Clone)]
struct CustomEvent {
    first_name: String,
    last_name: String,
}

#[derive(Serialize, Clone)]
struct CustomOutput {
    message: String,
}

fn main() {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();
    lambda!(my_handler);
}

fn my_handler(event: SqsEvent, _ctx: Context) -> Result<CustomOutput, HandlerError> {
    let mut email_ids = Vec::new();
    for record in event.records.into_iter() {
        let record_body = record.body.clone();
        if let Some(pointer) = EmailPointerMessage::from_message(record.into()) {
            email_ids.push(pointer.email_id);
        } else {
            info!("Could not parse email_id from {:?}", record_body);
        }
    }
    Ok(CustomOutput {
        message: format!("Hello {:?}!", email_ids),
    })
}

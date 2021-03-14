pub mod attribute_value_wrapper;
mod client;
mod dynamo;
mod email_message;
mod error;
mod queue;

pub use crate::client::Client;
pub use crate::queue::get_sqs_email_messages;

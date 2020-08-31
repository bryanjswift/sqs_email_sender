pub mod attribute_value_wrapper;
mod client;
mod dynamo;
pub mod email_message;
mod error;
pub mod queue;

pub use client::Client;
pub use dynamo::{get_email_message, set_email_status};
pub use error::UpdateError;

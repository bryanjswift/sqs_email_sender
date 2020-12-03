mod de;
mod dynamo;
mod error;

pub use de::from_hashmap;
pub use dynamo::{get_email_message, set_email_status, StatusTransition};

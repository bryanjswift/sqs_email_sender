mod de;
mod dynamo;
mod error;

pub use de::from_hashmap;
pub use dynamo::{get_email_message, set_email_to_sending, set_email_to_sent};

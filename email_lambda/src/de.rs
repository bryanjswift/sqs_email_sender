use rusoto_sqs::{Message, MessageAttributeValue};
use serde::Deserialize;

/// Implementation copied from `rusoto_sqs` at 0.45.0. Originally intended for use with [`serde`
/// remote (de)serialization](https://serde.rs/remote-derive.html) but ran into problems with the
/// `MessageAttributeValue` deserialization.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageDef {
    pub attributes: Option<::std::collections::HashMap<String, String>>,
    pub body: Option<String>,
    pub md5_of_body: Option<String>,
    pub md5_of_message_attributes: Option<String>,
    pub message_attributes: Option<::std::collections::HashMap<String, MessageAttributeValue>>,
    pub message_id: Option<String>,
    pub receipt_handle: Option<String>,
}

/// Create a `rusoto_sqs::Message` instance from a `MessageDef`. These structs should be equivalent
/// so it should simply be a matter of reassigning values.
impl From<MessageDef> for Message {
    /// Assign values from `MessageDef` to the attributes of `Message`.
    fn from(message: MessageDef) -> Self {
        Message {
            attributes: message.attributes,
            body: message.body,
            md5_of_body: message.md5_of_body,
            md5_of_message_attributes: message.md5_of_message_attributes,
            message_attributes: message.message_attributes,
            message_id: message.message_id,
            receipt_handle: message.receipt_handle,
        }
    }
}

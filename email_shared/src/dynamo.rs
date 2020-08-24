use rusoto_core::RusotoError;
use rusoto_dynamodb::{AttributeValue, GetItemError, GetItemInput, GetItemOutput};
use std::convert::TryFrom;

use crate::attribute_value_wrapper::DynamoItemWrapper;
use crate::email_message::{EmailMessage, ParseEmailMessageCode};
use crate::queue::EmailPointerMessage;

impl From<&EmailPointerMessage> for GetItemInput {
    fn from(message: &EmailPointerMessage) -> Self {
        let email_id_attribute = AttributeValue {
            s: Some(message.email_id.clone()),
            ..AttributeValue::default()
        };
        let mut input = GetItemInput::default();
        input
            .key
            .insert(String::from("EmailId"), email_id_attribute);
        input
    }
}

fn extract_email_field(
    wrapper: &DynamoItemWrapper,
    field: &str,
) -> Result<String, ParseEmailMessageCode> {
    wrapper.s(
        field,
        ParseEmailMessageCode::RecordMissingField(field.into()),
    )
}

impl TryFrom<GetItemOutput> for EmailMessage {
    type Error = ParseEmailMessageCode;

    fn try_from(data: GetItemOutput) -> Result<Self, Self::Error> {
        let item = data.item.ok_or(ParseEmailMessageCode::RecordNotFound)?;
        let wrapper = DynamoItemWrapper::new(item);
        let email_id = extract_email_field(&wrapper, "EmailId")?;
        let subject = extract_email_field(&wrapper, "Subject")?;
        Ok(EmailMessage {
            email_id,
            subject,
            ..EmailMessage::default()
        })
    }
}

impl From<RusotoError<GetItemError>> for ParseEmailMessageCode {
    fn from(_error: RusotoError<GetItemError>) -> ParseEmailMessageCode {
        ParseEmailMessageCode::RecordUnreachable
    }
}

#[cfg(test)]
mod try_from {
    use super::*;
    use rusoto_dynamodb::AttributeValue;
    use std::collections::HashMap;

    #[test]
    fn fails_on_empty_result() {
        let output = GetItemOutput {
            consumed_capacity: None,
            item: None,
        };
        match EmailMessage::try_from(output) {
            Ok(_) => panic!("Should not have parsed."),
            Err(code) => assert_eq!(code, ParseEmailMessageCode::RecordNotFound),
        };
    }

    #[test]
    fn fails_missing_id() {
        let attrs = HashMap::new();
        let item = Some(attrs);
        let output = GetItemOutput {
            consumed_capacity: None,
            item,
        };
        match EmailMessage::try_from(output) {
            Ok(_) => panic!("Should not have parsed."),
            Err(code) => assert_eq!(
                code,
                ParseEmailMessageCode::RecordMissingField("EmailId".into())
            ),
        };
    }

    #[test]
    fn fails_missing_subject() {
        let mut attrs = HashMap::new();
        attrs.insert(
            "EmailId".into(),
            AttributeValue {
                s: Some("foo".into()),
                ..AttributeValue::default()
            },
        );
        let item = Some(attrs);
        let output = GetItemOutput {
            consumed_capacity: None,
            item,
        };
        match EmailMessage::try_from(output) {
            Ok(_) => panic!("Should not have parsed."),
            Err(code) => assert_eq!(
                code,
                ParseEmailMessageCode::RecordMissingField("Subject".into())
            ),
        };
    }
}

use rusoto_core::RusotoError;
use rusoto_dynamodb::{
    AttributeValue, DynamoDb, DynamoDbClient, GetItemError, GetItemInput, GetItemOutput,
};
use std::convert::TryFrom;

use crate::attribute_value_wrapper::{AttributeValueMap, DynamoItemWrapper};
use crate::email_message::{EmailMessage, EmailStatus, ParseEmailMessageCode};
use crate::queue::EmailPointerMessage;

/// Get email data from Dynamo DB and then parse it into an `EmailMessage`. Uses the given
/// `DynamoDbClient` and attempts to get the item from the given `table_name`. Errors from they
/// Dynamo DB service are converted into `ParseEmailMessageCode`.
pub async fn get_email_message(
    dynamodb: &DynamoDbClient,
    table_name: &str,
    message: &EmailPointerMessage,
) -> Result<EmailMessage, ParseEmailMessageCode> {
    let input = GetItemInput {
        key: AttributeValueMap::with_entry("EmailId", message.email_id.clone()),
        table_name: table_name.into(),
        ..GetItemInput::default()
    };
    dynamodb
        .get_item(input)
        .await
        .map_err(ParseEmailMessageCode::from)
        .and_then(EmailMessage::try_from)
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
        Ok(EmailMessage {
            email_id: extract_email_field(&wrapper, "EmailId")?,
            subject: extract_email_field(&wrapper, "Subject")?,
            status: EmailStatus::from(extract_email_field(&wrapper, "Status")?.as_ref()),
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

    #[test]
    fn fails_missing_status() {
        let mut attrs = HashMap::new();
        attrs.insert(
            "EmailId".into(),
            AttributeValue {
                s: Some("Test EmailId".into()),
                ..AttributeValue::default()
            },
        );
        attrs.insert(
            "Subject".into(),
            AttributeValue {
                s: Some("Test Subject".into()),
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
                ParseEmailMessageCode::RecordMissingField("Status".into())
            ),
        };
    }

    #[test]
    fn succeeds() {
        let mut attrs = HashMap::new();
        attrs.insert(
            "EmailId".into(),
            AttributeValue {
                s: Some("Test EmailId".into()),
                ..AttributeValue::default()
            },
        );
        attrs.insert(
            "Subject".into(),
            AttributeValue {
                s: Some("Test Subject".into()),
                ..AttributeValue::default()
            },
        );
        attrs.insert(
            "Status".into(),
            AttributeValue {
                s: Some("Pending".into()),
                ..AttributeValue::default()
            },
        );
        let item = Some(attrs);
        let output = GetItemOutput {
            consumed_capacity: None,
            item,
        };
        match EmailMessage::try_from(output) {
            Ok(email) => {
                assert_eq!(&email.email_id, "Test EmailId");
                assert_eq!(&email.subject, "Test Subject");
                assert_eq!(email.status, EmailStatus::Pending);
            }
            Err(_) => panic!("Should have parsed."),
        };
    }
}

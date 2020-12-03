use rusoto_dynamodb::{DynamoDb, DynamoDbClient, GetItemInput, GetItemOutput, UpdateItemInput};
use std::convert::TryFrom;

use crate::attribute_value_wrapper::AttributeValueMap;
use crate::dynamo::error::DeserializeError;
use crate::email_message::{EmailMessage, EmailStatus};
use crate::error::{GetError, UpdateError};
use crate::queue::EmailPointerMessage;

/// Get email data from Dynamo DB and then parse it into an `EmailMessage`. Uses the given
/// `DynamoDbClient` and attempts to get the item from the given `table_name`. Errors from they
/// Dynamo DB service are converted into `GetError`.
pub async fn get_email_message(
    dynamodb: &DynamoDbClient,
    table_name: &str,
    message: &EmailPointerMessage,
) -> Result<EmailMessage, GetError> {
    let input = GetItemInput {
        key: AttributeValueMap::with_entry("EmailId", message.email_id.clone()),
        table_name: table_name.into(),
        ..GetItemInput::default()
    };
    dynamodb
        .get_item(input)
        .await
        .map_err(GetError::from)
        .and_then(EmailMessage::try_from)
}

/// Update the `EmailStatus` of the Dynamo record identified by `pointer` based on the `FromTo`
/// structure.
pub async fn set_email_status(
    dynamodb: &DynamoDbClient,
    table_name: &str,
    message: &EmailPointerMessage,
    args: FromTo,
) -> Result<(), UpdateError> {
    let FromTo(current_status, next_status) = args;
    let input = UpdateItemInput {
        condition_expression: Some("EmailStatus = :expected".to_owned()),
        expression_attribute_values: Some(AttributeValueMap::with_entries(vec![
            (":expected".into(), current_status.to_string()),
            (":next".into(), next_status.to_string()),
        ])),
        key: AttributeValueMap::with_entry("EmailId", message.email_id.clone()),
        table_name: table_name.into(),
        update_expression: Some("SET EmailStatus = :next".to_owned()),
        ..UpdateItemInput::default()
    };
    dynamodb
        .update_item(input)
        .await
        .map_err(UpdateError::from)
        .and_then(|_| Ok(()))
}

#[derive(Clone, Copy, Debug)]
pub struct FromTo(pub EmailStatus, pub EmailStatus);

impl TryFrom<GetItemOutput> for EmailMessage {
    type Error = GetError;

    fn try_from(data: GetItemOutput) -> Result<Self, Self::Error> {
        let item = data.item.ok_or(GetError::RecordNotFound)?;
        super::from_hashmap(item).map_err(|e| match e {
            DeserializeError::FieldMissing(field) => GetError::PropertyMissing(field),
            _ => GetError::ParseError(e.to_string()),
        })
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
            Err(code) => assert_eq!(code, GetError::RecordNotFound),
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
            Err(code) => assert_eq!(code, GetError::PropertyMissing("EmailId".into())),
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
        attrs.insert(
            "EmailStatus".into(),
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
            Ok(_) => panic!("Should not have parsed."),
            Err(code) => assert_eq!(code, GetError::PropertyMissing("Subject".into())),
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
            Err(code) => assert_eq!(code, GetError::PropertyMissing("EmailStatus".into())),
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
            "EmailStatus".into(),
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

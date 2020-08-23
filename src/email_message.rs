use crate::attribute_value_wrapper::DynamoItemWrapper;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{GetItemError, GetItemOutput};
use std::convert::TryFrom;

/// A `Recipient` represents an address to which a message will be sent.
type Recipient = String;

#[derive(Clone, Debug)]
enum EmailStatus {
    Pending,
    Sending,
    Sent,
    Unknown,
}

impl Default for EmailStatus {
    fn default() -> Self {
        EmailStatus::Pending
    }
}

impl From<&str> for EmailStatus {
    fn from(status: &str) -> Self {
        match status {
            "Pending" => EmailStatus::Pending,
            "Sending" => EmailStatus::Sending,
            "Sent" => EmailStatus::Sent,
            _ => EmailStatus::Unknown,
        }
    }
}

/// An attachment to an `EmailMessage`.
#[derive(Clone, Debug, Default)]
struct EmailMessageAttachment {
    /// base64 encoded contents of the message.
    body: String,
    /// File name of the attached `body`.
    name: String,
    /// MIME type of the `body`.
    content_type: String,
    /// byte size of the `body`.
    size: i32,
    /// Etag of the file retrieved from the webserver and included as `body`.
    e_tag: String,
    /// Last modified date of the file retrieved from the webserver and included as `body`.
    last_modified: String,
}

/// Represents data to be sent as an email via mail delivery services.
#[derive(Clone, Debug, Default)]
pub struct EmailMessage {
    /// Attachments to include with the email message.
    attachments: Vec<EmailMessageAttachment>,
    /// The HTML email body.
    body_html: String,
    /// The TXT email body.
    body_text: String,
    /// Identifier of the email.
    email_id: String,
    /// Count of times sending this email has failed.
    failed_count: i32,
    /// Provider through which the email was sent.
    provider: String,
    /// Response from the provider after sending the message successfully.
    provider_response: Option<String>,
    /// List of `Recipient` to BCC.
    recipients_bcc: Vec<Recipient>,
    /// List of `Recipient` to CC.
    recipients_cc: Vec<Recipient>,
    /// List of `Recipient` in TO.
    recipients_to: Vec<Recipient>,
    /// The FROM address.
    sender: Recipient,
    /// Count of times this email has sent successfully.
    sent_count: i32,
    /// DateTime of first successful email send.
    sent_at: Option<String>,
    /// Last known state of the message.
    status: EmailStatus,
    /// SUBJECT of the email.
    subject: String,
    /// DateTime indicating the last time this record was updated.
    updated_at: String,
}

impl TryFrom<GetItemOutput> for EmailMessage {
    type Error = ParseEmailMessageCode;

    fn try_from(data: GetItemOutput) -> Result<Self, Self::Error> {
        let item = data.item.ok_or(ParseEmailMessageCode::RecordNotFound)?;
        let wrapper = DynamoItemWrapper::new(item);
        let email_id = wrapper.s("EmailId", ParseEmailMessageCode::RecordMissingId)?;
        let subject = wrapper.s("Subject", ParseEmailMessageCode::RecordMissingSubject)?;
        Ok(EmailMessage {
            email_id,
            subject,
            ..EmailMessage::default()
        })
    }
}

/// Possible errors while attempting to pull fields out of `GetItemOutput`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseEmailMessageCode {
    /// The specified record did not exist.
    RecordNotFound,
    /// The record was missing an id field.
    RecordMissingId,
    /// The record was missing a subject field.
    RecordMissingSubject,
    /// The service could not be reached to retrieve a record. This indicates an underlying
    /// problem, check the logs.
    RecordUnreachable,
}

impl From<RusotoError<GetItemError>> for ParseEmailMessageCode {
    fn from(_error: RusotoError<GetItemError>) -> ParseEmailMessageCode {
        ParseEmailMessageCode::RecordUnreachable
    }
}

impl From<ParseEmailMessageCode> for String {
    fn from(code: ParseEmailMessageCode) -> String {
        format!("{}", code)
    }
}

impl std::fmt::Display for ParseEmailMessageCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod from {
    use super::*;

    #[test]
    fn changes_code_to_string() {
        let output = String::from(ParseEmailMessageCode::RecordUnreachable);
        assert_eq!(output, "RecordUnreachable");
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
            Err(code) => assert_eq!(code, ParseEmailMessageCode::RecordMissingId),
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
            Err(code) => assert_eq!(code, ParseEmailMessageCode::RecordMissingSubject),
        };
    }
}

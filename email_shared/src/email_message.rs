use thiserror::Error;

/// A `Recipient` represents an address to which a message will be sent.
type Recipient = String;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmailStatus {
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
pub struct EmailMessageAttachment {
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
    pub attachments: Vec<EmailMessageAttachment>,
    /// The HTML email body.
    pub body_html: String,
    /// The TXT email body.
    pub body_text: String,
    /// Identifier of the email.
    pub email_id: String,
    /// Provider through which the email was sent.
    pub provider: String,
    /// Response from the provider after sending the message successfully.
    pub provider_response: Option<String>,
    /// List of `Recipient` to BCC.
    pub recipients_bcc: Vec<Recipient>,
    /// List of `Recipient` to CC.
    pub recipients_cc: Vec<Recipient>,
    /// List of `Recipient` in TO.
    pub recipients_to: Vec<Recipient>,
    /// The FROM address.
    pub sender: Recipient,
    /// DateTime of first successful email send.
    pub sent_at: Option<String>,
    /// Last known state of the message.
    pub status: EmailStatus,
    /// SUBJECT of the email.
    pub subject: String,
    /// DateTime indicating the last time this record was updated.
    pub updated_at: String,
}

/// Possible errors while attempting to pull fields out of `GetItemOutput`.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ParseEmailMessageCode {
    /// The specified record did not exist.
    #[error("The specified record did not exist.")]
    RecordNotFound,
    /// The record was missing a field.
    #[error("The record was missing the `{0}` attribute.")]
    RecordMissingField(String),
    /// The service could not be reached to retrieve a record. This indicates an underlying
    /// problem, check the logs.
    #[error("An error occurred attempting to access the record.")]
    RecordUnreachable,
}

impl From<ParseEmailMessageCode> for String {
    fn from(code: ParseEmailMessageCode) -> String {
        format!("{}", code)
    }
}

#[cfg(test)]
mod from {
    use super::*;

    #[test]
    fn changes_code_to_string() {
        let output = String::from(ParseEmailMessageCode::RecordUnreachable);
        assert_eq!(output, "An error occurred attempting to access the record.");
    }
}

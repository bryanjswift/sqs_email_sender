use crate::dynamo::{get_email_message, set_email_to_sending, set_email_to_sent};
use crate::email_message::{EmailMessage, EmailStatus};
use crate::error::ProcessError;
use crate::queue::EmailPointerMessage;
use rusoto_dynamodb::DynamoDbClient;
use rusoto_sqs::{DeleteMessageBatchRequestEntry, Message};
use std::convert::TryFrom;
use tracing::{event, span, Instrument, Level};

/// Hold references to external service clients so they only need to be allocated once.
pub struct Client<'a> {
    /// Connection to DynamoDB
    dynamodb: &'a DynamoDbClient,
    /// DynamoDB table from which email data will be read.
    table_name: &'a str,
}

impl Client<'_> {
    pub fn new<'a>(dynamodb: &'a DynamoDbClient, table_name: &'a str) -> Client<'a> {
        Client {
            dynamodb,
            table_name,
        }
    }

    #[tracing::instrument(skip(messages), level = Level::INFO)]
    pub async fn process_messages<I>(&self, messages: I) -> Vec<DeleteMessageBatchRequestEntry>
    where
        I: IntoIterator<Item = Message>,
    {
        let mut processed_message_handles = Vec::new();
        for message in messages {
            let message_span =
                span!(Level::INFO, "process_message", message_id = ?&message.message_id);
            match self.process_message(message).instrument(message_span).await {
                Ok(pointer) | Err(ProcessError::Skip(pointer)) => {
                    processed_message_handles.push(DeleteMessageBatchRequestEntry::from(&pointer));
                }
                Err(ProcessError::SkipMessage(message)) => {
                    processed_message_handles.push(DeleteMessageBatchRequestEntry {
                        id: message.message_id.unwrap(),
                        receipt_handle: message.receipt_handle.unwrap(),
                    });
                }
                Err(ProcessError::Retry) => {
                    continue;
                }
            }
        }
        processed_message_handles
    }

    /// For the given `Message` attempt to extract an `EmailPointerMessage` and transmit the associated
    /// `EmailMessage` with the declared sending service.
    async fn process_message(&self, message: Message) -> Result<EmailPointerMessage, ProcessError> {
        let dynamodb = self.dynamodb;
        let table_name = self.table_name;
        // Which errors mean try again and which errors mean skip message?
        // 1. Parse email_id from SQS message
        let pointer = EmailPointerMessage::try_from(message.clone());
        let pointer = match pointer {
            Ok(record) => record,
            Err(msg) => {
                event!(Level::ERROR, error = msg, "pointer parse failure");
                return Err(ProcessError::SkipMessage(message));
            }
        };
        // Create logger for this record
        // 2. Get email data from dynamo db table
        // 3. Parse dynamo data into object for sending
        event!(Level::INFO, %table_name, "get email");
        let email = get_email_message(dynamodb, table_name, &pointer).await;
        // 4. If status of email is not `EmailStatus::Pending` log a warning and skip sending. The
        //    message to remove will automatically be created.
        let email = match email {
            Ok(mail) if mail.status != EmailStatus::Pending => {
                event!(Level::WARN, email_status = %mail.status, "email not {}", EmailStatus::Pending);
                // See 8.
                // Skipping doesn't work unless the pointer is recorded as an entry to be deleted.
                return Err(ProcessError::Skip(pointer));
            }
            Ok(mail) => mail,
            Err(error) => {
                event!(Level::ERROR, %error, "get email failed");
                return Err(ProcessError::Retry);
            }
        };
        // 5. Update the message status in dynamo so that a second receiver for this message will
        //    not try to send the same email
        let update_result = set_email_to_sending(dynamodb, table_name, &pointer).await;
        if let Err(error) = update_result {
            event!(Level::ERROR, %error, "update email status to Sending failed");
            return Err(ProcessError::Retry);
        }
        // 6. TODO: Send the message
        event!(Level::INFO, email_status = %email.status, "start email transmit");
        let send_result = Client::send_email(email).await;
        if let Err(error) = send_result {
            event!(Level::ERROR, %error, "send email failed");
            return Err(ProcessError::Retry);
        }
        // 7. Update the message status in dynamo to sent
        let update_result = set_email_to_sent(dynamodb, table_name, &pointer).await;
        if let Err(error) = update_result {
            event!(Level::ERROR, %error, "update email failed");
            return Err(ProcessError::Retry);
        }
        // 8. Messages are automatically removed from the queue if lambda succeeds. Keep track of
        //    the successfully processed messages so in the event of partial (or total) batch
        //    failure the successful messages can be deleted but the errored messages will get
        //    redelivered.
        Ok(pointer)
    }

    async fn send_email(email: EmailMessage) -> Result<(), String> {
        event!(Level::INFO, email = ?email, "send_email");
        Err("Unimplemented".into())
    }
}

impl<'a> std::fmt::Debug for Client<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").finish()
    }
}

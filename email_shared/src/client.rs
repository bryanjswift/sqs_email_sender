use crate::email_message::{EmailMessage, ParseEmailMessageCode};
use crate::queue::EmailPointerMessage;
use log::{error, info};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, GetItemInput};
use rusoto_sqs::Message;
use std::convert::TryFrom;

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

    pub async fn process_messages(&self, messages: Vec<Message>) -> Vec<EmailPointerMessage> {
        info!("Process messages, {:?}", messages);
        let mut processed_message_handles = Vec::new();
        for message in messages {
            match self.process_message(message).await {
                Ok(id_message) => processed_message_handles.push(id_message),
                Err(_) => (), // TODO: This needs to at least log the error
            }
        }
        processed_message_handles
    }

    async fn process_message(&self, message: Message) -> Result<EmailPointerMessage, String> {
        let pointer = EmailPointerMessage::try_from(message)?;
        let email_message = self.get_email_message(&pointer).await;
        let send_result = match email_message {
            Ok(email) => Client::send_email(email).await,
            Err(error) => {
                error!("process_message: {}: {}", &pointer, error);
                Err("Unable to Parse Email".into())
            }
        };
        match send_result {
            Ok(_) => Ok(pointer),
            Err(msg) => Err(msg),
        }
    }

    async fn get_email_message(
        &self,
        message: &EmailPointerMessage,
    ) -> Result<EmailMessage, ParseEmailMessageCode> {
        let mut input = GetItemInput::from(message);
        input.table_name = self.table_name.into();
        self.dynamodb
            .get_item(input)
            .await
            .map_err(ParseEmailMessageCode::from)
            .and_then(EmailMessage::try_from)
    }

    async fn send_email(email: EmailMessage) -> Result<(), String> {
        info!("send_email: {:?}", email);
        Err("Unimplemented".into())
    }
}

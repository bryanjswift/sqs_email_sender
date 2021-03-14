use crate::queue::EmailPointerMessage;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{GetItemError, UpdateItemError};
use rusoto_sqs::Message;
use thiserror::Error;

/// Possible errors from updating an item in DynamoDB.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum UpdateError {
    #[error("ConditionalCheckFailed({0})")]
    ConditionalCheckFailed(String),
    #[error("InternalServerError({0})")]
    InternalServerError(String),
    #[error("ItemCollectionSizeLimitExceeded({0})")]
    ItemCollectionSizeLimitExceeded(String),
    #[error("ProvisionedThroughputExceeded({0})")]
    ProvisionedThroughputExceeded(String),
    #[error("RequestLimitExceeded({0})")]
    RequestLimitExceeded(String),
    #[error("ResourceNotFound({0})")]
    ResourceNotFound(String),
    #[error("RusotoError({0})")]
    ServiceError(String),
    #[error("TransactionConflict({0})")]
    TransactionConflict(String),
}

impl From<UpdateItemError> for UpdateError {
    fn from(error: UpdateItemError) -> Self {
        match error {
            UpdateItemError::ConditionalCheckFailed(msg) => Self::ConditionalCheckFailed(msg),
            UpdateItemError::InternalServerError(msg) => Self::InternalServerError(msg),
            UpdateItemError::ItemCollectionSizeLimitExceeded(msg) => {
                Self::ItemCollectionSizeLimitExceeded(msg)
            }
            UpdateItemError::ProvisionedThroughputExceeded(msg) => {
                Self::ProvisionedThroughputExceeded(msg)
            }
            UpdateItemError::RequestLimitExceeded(msg) => Self::RequestLimitExceeded(msg),
            UpdateItemError::ResourceNotFound(msg) => Self::ResourceNotFound(msg),
            UpdateItemError::TransactionConflict(msg) => Self::TransactionConflict(msg),
        }
    }
}

impl From<RusotoError<UpdateItemError>> for UpdateError {
    fn from(error: RusotoError<UpdateItemError>) -> Self {
        match error {
            RusotoError::Service(service_error) => Self::from(service_error),
            rusoto_error => Self::ServiceError(format!("{}", rusoto_error)),
        }
    }
}

/// Possible errors while attempting to retrieve an item from DynamoDB.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum GetError {
    #[error("InternalServerError({0})")]
    InternalServerError(String),
    #[error("ParseError({0})")]
    ParseError(String),
    #[error("PropertyMissiong({0})")]
    PropertyMissing(String),
    #[error("ProvisionedThroughputExceeded({0})")]
    ProvisionedThroughputExceeded(String),
    #[error("RecordNotFound")]
    RecordNotFound,
    #[error("RequestLimitExceeded({0})")]
    RequestLimitExceeded(String),
    #[error("ResourceNotFound({0})")]
    ResourceNotFound(String),
    #[error("RusotoError({0})")]
    ServiceError(String),
}

impl From<GetItemError> for GetError {
    fn from(error: GetItemError) -> Self {
        match error {
            GetItemError::InternalServerError(msg) => Self::InternalServerError(msg),
            GetItemError::ProvisionedThroughputExceeded(msg) => {
                Self::ProvisionedThroughputExceeded(msg)
            }
            GetItemError::RequestLimitExceeded(msg) => Self::RequestLimitExceeded(msg),
            GetItemError::ResourceNotFound(msg) => Self::ResourceNotFound(msg),
        }
    }
}

impl From<RusotoError<GetItemError>> for GetError {
    fn from(error: RusotoError<GetItemError>) -> Self {
        match error {
            RusotoError::Service(service_error) => Self::from(service_error),
            rusoto_error => Self::ServiceError(format!("{}", rusoto_error)),
        }
    }
}

/// Possible errors processing an SQS `Message` as an `EmailIdMessage`.
#[derive(Clone, Debug, Error)]
pub enum ProcessError {
    /// Indicates some necessary operation could not be completed due to a temporary condition the
    /// processing the `Message` should be attempted again.
    #[error("Retry")]
    Retry,
    /// Indicates processing has skipped sending the email associated with `EmailPointerMessage`
    /// and the `Message` should not be reprocessed later.
    #[error("Skip({0})")]
    Skip(EmailPointerMessage),
    /// Processing result indicating the SQS `Message` must be skipped and can not be handled. This
    /// is not a temporary or ephemeral error. Reprocessing the `Message` will also fail.
    #[error("SkipMessage({0:?})")]
    SkipMessage(Message),
}

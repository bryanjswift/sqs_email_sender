use rusoto_core::RusotoError;
use rusoto_dynamodb::UpdateItemError;
use thiserror::Error;

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
            UpdateItemError::ConditionalCheckFailed(msg) => {
                UpdateError::ConditionalCheckFailed(msg)
            }
            UpdateItemError::InternalServerError(msg) => UpdateError::InternalServerError(msg),
            UpdateItemError::ItemCollectionSizeLimitExceeded(msg) => {
                UpdateError::ItemCollectionSizeLimitExceeded(msg)
            }
            UpdateItemError::ProvisionedThroughputExceeded(msg) => {
                UpdateError::ProvisionedThroughputExceeded(msg)
            }
            UpdateItemError::RequestLimitExceeded(msg) => UpdateError::RequestLimitExceeded(msg),
            UpdateItemError::ResourceNotFound(msg) => UpdateError::ResourceNotFound(msg),
            UpdateItemError::TransactionConflict(msg) => UpdateError::TransactionConflict(msg),
        }
    }
}

impl From<RusotoError<UpdateItemError>> for UpdateError {
    fn from(error: RusotoError<UpdateItemError>) -> Self {
        match error {
            RusotoError::Service(service_error) => UpdateError::from(service_error),
            rusoto_error => UpdateError::ServiceError(format!("{}", rusoto_error)),
        }
    }
}

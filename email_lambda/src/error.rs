use email_shared::GetError;
use lambda_runtime::error::{HandlerError, LambdaErrorExt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailHandlerError {
    BatchFailure,
    PartialBatchFailure,
    SqsDeleteFailed,
}

impl Default for EmailHandlerError {
    fn default() -> Self {
        EmailHandlerError::BatchFailure
    }
}

impl LambdaErrorExt for EmailHandlerError {
    fn error_type(&self) -> &str {
        match self {
            Self::BatchFailure => "BatchFailure",
            Self::PartialBatchFailure => "PartialBatchFailure",
            Self::SqsDeleteFailed => "SqsDeleteFailed",
        }
    }
}

impl std::fmt::Display for EmailHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EmailHandlerError {}

impl From<GetError> for EmailHandlerError {
    fn from(_value: GetError) -> Self {
        EmailHandlerError::PartialBatchFailure
    }
}

impl From<EmailHandlerError> for HandlerError {
    fn from(error: EmailHandlerError) -> Self {
        Self::new(error)
    }
}

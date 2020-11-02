use email_shared::GetError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailHandlerError {
    InitializationFailure,
    BatchFailure,
    PartialBatchFailure,
    SqsDeleteFailed,
}

impl Default for EmailHandlerError {
    fn default() -> Self {
        EmailHandlerError::BatchFailure
    }
}

impl std::fmt::Display for EmailHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EmailHandlerError {}

impl From<std::env::VarError> for EmailHandlerError {
    fn from(_value: std::env::VarError) -> Self {
        Self::InitializationFailure
    }
}

impl From<GetError> for EmailHandlerError {
    fn from(_value: GetError) -> Self {
        EmailHandlerError::PartialBatchFailure
    }
}

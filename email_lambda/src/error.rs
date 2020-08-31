use email_shared::email_message::ParseEmailMessageCode;
use lambda_runtime::error::{HandlerError, LambdaErrorExt};

#[derive(Clone, Debug, Default)]
pub struct EmailHandlerError {
    err_type: Option<String>,
}

impl LambdaErrorExt for EmailHandlerError {
    fn error_type(&self) -> &str {
        if let Some(error_type) = &self.err_type {
            error_type
        } else {
            "EmailHandlerError"
        }
    }
}

impl std::fmt::Display for EmailHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EmailHandlerError {}

impl From<ParseEmailMessageCode> for EmailHandlerError {
    fn from(value: ParseEmailMessageCode) -> Self {
        EmailHandlerError {
            err_type: Some(format!("{}", value)),
        }
    }
}

impl From<EmailHandlerError> for HandlerError {
    fn from(error: EmailHandlerError) -> Self {
        Self::new(error)
    }
}

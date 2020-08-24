use lambda_runtime::{error::HandlerError, lambda, Context};
use log::error;
use serde::{Deserialize, Serialize};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};

#[derive(Deserialize, Clone)]
struct CustomEvent {
    first_name: String,
    last_name: String,
}

#[derive(Serialize, Clone)]
struct CustomOutput {
    message: String,
}

fn main() {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();
    lambda!(my_handler);
}

fn my_handler(e: CustomEvent, _ctx: Context) -> Result<CustomOutput, HandlerError> {
    if e.first_name == "" {
        error!("Empty first name");
    }
    Ok(CustomOutput {
        message: format!("Hello, {}!", e.first_name),
    })
}

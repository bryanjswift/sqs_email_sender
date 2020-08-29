pub mod attribute_value_wrapper;
mod client;
mod dynamo;
pub mod email_message;
pub mod queue;

pub use client::Client;
pub use dynamo::get_email_message;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

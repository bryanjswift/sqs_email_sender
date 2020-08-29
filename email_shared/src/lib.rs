pub mod attribute_value_wrapper;
mod client;
pub mod dynamo;
pub mod email_message;
pub mod queue;

pub use client::Client;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod attribute_value_wrapper;
pub mod email_message;
pub mod queue;
pub mod sqs_email_messages;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

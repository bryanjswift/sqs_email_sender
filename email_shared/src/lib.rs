pub mod attribute_value_wrapper;
pub mod dynamo;
pub mod email_message;
pub mod queue;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

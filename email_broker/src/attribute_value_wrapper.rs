use rusoto_dynamodb::AttributeValue;
use std::collections::HashMap;

/// Wrap the `item` representation provided by `rusoto_dynamodb::GetItemOutput` in order to more
/// conveniently access the properties of an `AttributeValue` hiddent behind an arbitrary `&str`
/// key.
///
/// # Examples
///
/// ```
/// let item: HashMap<String, AttributeValue> = HashMap::new();
/// let wrapper = DynamoItemWrapper::new(item);
/// let email_id = wrapper.s("EmailId", FetchEmailMessageCode::RecordMissingId);
/// assert!(email_id.is_err());
/// ```
pub struct DynamoItemWrapper {
    item: HashMap<String, AttributeValue>,
}

impl DynamoItemWrapper {
    /// Create a new `DynamoItemWrapper`.
    ///
    /// The `DynamoItemWrapper` is entirely dependent on the given `HashMap` for values.
    ///
    /// # Examples
    ///
    /// ```
    /// let item: HashMap<String, AttributeValue> = HashMap::new();
    /// let wrapper = DynamoItemWrapper::new(item);
    /// ```
    pub fn new(item: HashMap<String, AttributeValue>) -> Self {
        DynamoItemWrapper { item }
    }

    /// Try to retrieve an `AttributeValue` for `key` and then try to get the `String` value from
    /// the associated `AttributeValue`. If either retrieving an `AttributeValue` or getting a
    /// `String` value fails provide the given `error`.
    ///
    /// # Examples
    ///
    /// ```
    /// let item: HashMap<String, AttributeValue> = HashMap::new();
    /// let wrapper = DynamoItemWrapper::new(item);
    /// assert!(wrapper.s("foo", "bar"), Err("bar"));
    /// ```
    pub fn s<E>(&self, key: &str, error: E) -> Result<String, E> {
        self.item.get(key).and_then(|av| av.s.clone()).ok_or(error)
    }
}

#[cfg(test)]
mod s {
    use super::*;
    use std::collections::HashMap;

    const ERROR_MSG: &str = "error";
    const EMAIL_ID_KEY: &str = "EmailId";
    const EMAIL_ID_VALUE: &str = "foo";

    fn create_item() -> HashMap<String, AttributeValue> {
        let mut attributes = HashMap::new();
        attributes.insert(
            EMAIL_ID_KEY.into(),
            AttributeValue {
                s: Some(EMAIL_ID_VALUE.into()),
                ..AttributeValue::default()
            },
        );
        attributes
    }

    #[test]
    fn error_when_missing() {
        let wrapper = DynamoItemWrapper::new(create_item());
        assert_eq!(wrapper.s(EMAIL_ID_VALUE, ERROR_MSG), Err(ERROR_MSG));
    }

    #[test]
    fn ok_when_exists() {
        let wrapper = DynamoItemWrapper::new(create_item());
        assert_eq!(
            wrapper.s(EMAIL_ID_KEY, ERROR_MSG),
            Ok(EMAIL_ID_VALUE.into())
        );
    }
}

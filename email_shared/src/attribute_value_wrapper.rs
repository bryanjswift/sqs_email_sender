use rusoto_dynamodb::AttributeValue;
use std::collections::HashMap;

pub struct AttributeValueMap {}

type AttributeValueStringEntry = (String, String);

impl AttributeValueMap {
    /// Create a new `HashMap` of `AttributeValue` structs with one property identified by `key`.
    ///
    /// # Examples
    ///
    /// ```
    /// use email_shared::attribute_value_wrapper::AttributeValueMap;
    ///
    /// let item = AttributeValueMap::with_entry("foo", "bar".into());
    /// assert!(item.get("foo").unwrap().s == Some("bar".into()));
    /// assert!(item.get("other_foo") == None);
    /// ```
    pub fn with_entry(key: &str, value: String) -> HashMap<String, AttributeValue> {
        let mut attrs = HashMap::new();
        attrs.insert(
            key.into(),
            AttributeValue {
                s: Some(value),
                ..AttributeValue::default()
            },
        );
        attrs
    }

    /// Create a new `HashMap` of `AttributeValue` structs with string properties for each tuple of
    /// key, value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use email_shared::attribute_value_wrapper::AttributeValueMap;
    ///
    /// let item = AttributeValueMap::with_entries(vec![
    ///     (":expected".into(), "bar".into()),
    ///     (":next".into(), "Test Next".into()),
    /// ]);
    /// assert!(item.get(":expected").unwrap().s == Some("bar".into()));
    /// assert!(item.get(":next").unwrap().s == Some("Test Next".into()));
    /// assert!(item.get("other_foo") == None);
    /// ```
    pub fn with_entries<I>(entries: I) -> HashMap<String, AttributeValue>
    where
        I: IntoIterator<Item = AttributeValueStringEntry>,
    {
        let mut attrs = HashMap::new();
        for (key, value) in entries {
            attrs.insert(
                key,
                AttributeValue {
                    s: Some(value),
                    ..AttributeValue::default()
                },
            );
        }
        attrs
    }
}

/// Wrap the `item` representation provided by `rusoto_dynamodb::GetItemOutput` in order to more
/// conveniently access the properties of an `AttributeValue` hiddent behind an arbitrary `&str`
/// key.
///
/// # Examples
///
/// ```
/// use rusoto_dynamodb::AttributeValue;
/// use std::collections::HashMap;
/// use email_shared::attribute_value_wrapper::DynamoItemWrapper;
///
/// let item: HashMap<String, AttributeValue> = HashMap::new();
/// let wrapper = DynamoItemWrapper::new(item);
/// let email_id = wrapper.s("EmailId", "Error Message");
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
    /// use rusoto_dynamodb::AttributeValue;
    /// use std::collections::HashMap;
    /// use email_shared::attribute_value_wrapper::DynamoItemWrapper;
    ///
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
    /// use rusoto_dynamodb::AttributeValue;
    /// use std::collections::HashMap;
    /// use email_shared::attribute_value_wrapper::DynamoItemWrapper;
    ///
    /// let mut item: HashMap<String, AttributeValue> = HashMap::new();
    /// item.insert(
    ///     "foo".into(),
    ///     AttributeValue {
    ///         s: Some("My String".into()),
    ///         ..AttributeValue::default()
    ///     }
    /// );
    /// let wrapper = DynamoItemWrapper::new(item);
    /// assert!(wrapper.s("foo", "bar") == Ok("My String".into()));
    /// ```
    ///
    /// ```
    /// use rusoto_dynamodb::AttributeValue;
    /// use std::collections::HashMap;
    /// use email_shared::attribute_value_wrapper::DynamoItemWrapper;
    ///
    /// let item: HashMap<String, AttributeValue> = HashMap::new();
    /// let wrapper = DynamoItemWrapper::new(item);
    /// assert!(wrapper.s("foo", "bar") == Err("bar"));
    /// ```
    ///
    /// ```
    /// use rusoto_dynamodb::AttributeValue;
    /// use std::collections::HashMap;
    /// use email_shared::attribute_value_wrapper::DynamoItemWrapper;
    ///
    /// let mut item: HashMap<String, AttributeValue> = HashMap::new();
    /// item.insert("foo".into(), AttributeValue::default());
    /// let wrapper = DynamoItemWrapper::new(item);
    /// assert!(wrapper.s("foo", "bar") == Err("bar"));
    /// ```
    pub fn s<E>(&self, key: &str, error: E) -> Result<String, E> {
        self.item.get(key).and_then(|av| av.s.clone()).ok_or(error)
    }

    /// Try to retrieve an `AttributeValue` for `key` and then try to get the number value from the
    /// associated `AttributeValue`. If either retrieving an `AttributeValue` or getting a number
    /// value fails wrap the given `error` in `Err`. Because Dynamo DB transmits number values as
    /// strings the `Result` holds a `String` value.
    ///
    /// # Examples
    ///
    /// ## Get a numeric value
    ///
    /// ```
    /// use rusoto_dynamodb::AttributeValue;
    /// use std::collections::HashMap;
    /// use email_shared::attribute_value_wrapper::DynamoItemWrapper;
    ///
    /// let mut item: HashMap<String, AttributeValue> = HashMap::new();
    /// item.insert(
    ///     "foo".into(),
    ///     AttributeValue {
    ///         n: Some("123.45".into()),
    ///         ..AttributeValue::default()
    ///     }
    /// );
    /// let wrapper = DynamoItemWrapper::new(item);
    /// assert!(wrapper.n("foo", "bar") == Ok("123.45".into()));
    /// ```
    ///
    /// ## `Err` for non-existant attribute
    ///
    /// ```
    /// use rusoto_dynamodb::AttributeValue;
    /// use std::collections::HashMap;
    /// use email_shared::attribute_value_wrapper::DynamoItemWrapper;
    ///
    /// let item: HashMap<String, AttributeValue> = HashMap::new();
    /// let wrapper = DynamoItemWrapper::new(item);
    /// assert!(wrapper.n("foo", "bar") == Err("bar"));
    /// ```
    ///
    /// ## `Err` for empty number
    ///
    /// ```
    /// use rusoto_dynamodb::AttributeValue;
    /// use std::collections::HashMap;
    /// use email_shared::attribute_value_wrapper::DynamoItemWrapper;
    ///
    /// let mut item: HashMap<String, AttributeValue> = HashMap::new();
    /// item.insert("foo".into(), AttributeValue::default());
    /// let wrapper = DynamoItemWrapper::new(item);
    /// assert!(wrapper.n("foo", "bar") == Err("bar"));
    /// ```
    pub fn n<E>(&self, key: &str, error: E) -> Result<String, E> {
        self.item.get(key).and_then(|av| av.n.clone()).ok_or(error)
    }
}

#[cfg(test)]
mod s {
    use super::*;
    use std::collections::HashMap;

    const ERROR_MSG: &str = "error";
    const EMAIL_ID_KEY: &str = "EmailId";
    const EMAIL_ID_VALUE: &str = "foo";

    #[test]
    fn error_when_missing() {
        let attributes = HashMap::new();
        let wrapper = DynamoItemWrapper::new(attributes);
        assert_eq!(wrapper.s(EMAIL_ID_VALUE, ERROR_MSG), Err(ERROR_MSG));
    }

    #[test]
    fn error_when_wrong_type() {
        // Attribute has the correct value but under the wrong type
        let mut attributes = HashMap::new();
        attributes.insert(
            EMAIL_ID_KEY.into(),
            AttributeValue {
                n: Some(EMAIL_ID_VALUE.into()),
                ..AttributeValue::default()
            },
        );
        let wrapper = DynamoItemWrapper::new(attributes);
        assert_eq!(wrapper.s(EMAIL_ID_KEY, ERROR_MSG), Err(ERROR_MSG));
    }

    #[test]
    fn ok_when_exists() {
        let mut attributes = HashMap::new();
        attributes.insert(
            EMAIL_ID_KEY.into(),
            AttributeValue {
                s: Some(EMAIL_ID_VALUE.into()),
                ..AttributeValue::default()
            },
        );
        let wrapper = DynamoItemWrapper::new(attributes);
        assert_eq!(
            wrapper.s(EMAIL_ID_KEY, ERROR_MSG),
            Ok(EMAIL_ID_VALUE.into())
        );
    }
}

#[cfg(test)]
mod n {
    use super::*;
    use std::collections::HashMap;

    const ERROR_MSG: &str = "error";
    const EMAIL_ID_KEY: &str = "EmailId";
    const EMAIL_ID_VALUE: &str = "123.45";

    #[test]
    fn error_when_missing() {
        let attributes = HashMap::new();
        let wrapper = DynamoItemWrapper::new(attributes);
        assert_eq!(wrapper.n(EMAIL_ID_KEY, ERROR_MSG), Err(ERROR_MSG));
    }

    #[test]
    fn error_when_wrong_type() {
        // Attribute has the correct value but under the wrong type
        let mut attributes = HashMap::new();
        attributes.insert(
            EMAIL_ID_KEY.into(),
            AttributeValue {
                s: Some(EMAIL_ID_VALUE.into()),
                ..AttributeValue::default()
            },
        );
        let wrapper = DynamoItemWrapper::new(attributes);
        assert_eq!(wrapper.n(EMAIL_ID_KEY, ERROR_MSG), Err(ERROR_MSG));
    }

    #[test]
    fn ok_when_exists() {
        let mut attributes = HashMap::new();
        attributes.insert(
            EMAIL_ID_KEY.into(),
            AttributeValue {
                n: Some(EMAIL_ID_VALUE.into()),
                ..AttributeValue::default()
            },
        );
        let wrapper = DynamoItemWrapper::new(attributes);
        assert_eq!(
            wrapper.n(EMAIL_ID_KEY, ERROR_MSG),
            Ok(EMAIL_ID_VALUE.into())
        );
    }
}

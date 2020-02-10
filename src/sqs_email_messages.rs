use crate::email_id_message::EmailIdMessage;
use rusoto_sqs::Message;

#[derive(Debug)]
pub struct SqsEmailMessages {
    messages: Vec<Message>,
}

impl SqsEmailMessages {
    pub fn new(messages: Vec<Message>) -> SqsEmailMessages {
        SqsEmailMessages { messages: messages }
    }
}

impl Iterator for SqsEmailMessages {
    type Item = EmailIdMessage;

    /// Get the next email message identifier from a list of SQS `Message`s. If the current
    /// `Message` does not represent an email message identifier skip it and try the next one.
    ///
    /// Returns [`None`] when there are no `Message` instances left to try, once [`None`] is
    /// returned there will be no additional [`Some(EmailIdMessage)`] forthcoming.
    ///
    /// [`None`]: https://doc.rust-lang.org/stable/std/option/enum.Option.html#variant.None
    /// [`Some(Item)`]: https://doc.rust-lang.org/stable/std/option/enum.Option.html#variant.Some
    fn next(&mut self) -> Option<EmailIdMessage> {
        if self.messages.is_empty() {
            return None;
        }
        let message = self.messages.remove(0);
        let email = EmailIdMessage::from_message(message);
        match email {
            Some(item) => Some(item),
            None => self.next(),
        }
    }
}

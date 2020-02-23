# Email Service

The goal of the email service is to enable the decoupling of triggering email
notifications from the work of transmitting email to a third party service and
receiving callbacks from the services.

## Queue

The service polls an Amazon SQS queue looking for messages. The SQS messages
are expected to have a JSON body containing an `email_id` key. The `email_id`
is used to look up the email information in a database.

## Database

The `email_id` from the queue message is used to look up a record in an Amazon
DynamoDB table. The keys and values in the DynamoDB row are mapped to a
structure representing the data a third party email sending service needs to
transmit the message.

## Email Delivery Service(s)

No third party email sending service(s) are implemented yet.

## Environment Variables

AWS credentials are read from the environment by default. The email service
relies on [rusoto][rusoto] to access AWS. Rusoto provides a [`credential`
crate][credential] which is used to determine how AWS resources will be
accessed. The [`ChainProvider`][credential_chain_provider] currently defines
how AWS credentials are read.

[rusoto]: https://github.com/rusoto/rusoto
[credential]: https://crates.io/crates/rusoto_credential
[credential_chain_provider]: https://docs.rs/rusoto_credential/0.43.0-beta.1/rusoto_credential/struct.ChainProvider.html

- `REGION` defines the AWS region where the SQS queue and DynamoDB table are
  located.
- `QUEUE_URL` defines the SQS queue polled for messages.
- `TABLE_NAME` defines the name of the DynamoDB from which email messae data to
  send will be read.
- `DRY_RUN` when defined as the string `true` the queue will only be polled a
  single time and no email information will be transmitted to the email sending
  service(s).

## Development

This has been developed on MacOS using [Rust v1.41.0][rust-stable]. See [Rust
install docs][rust-install] for more detailed setup information.

[rust-stable]: https://www.rust-lang.org
[rust-install]: https://www.rust-lang.org/tools/install

### Prerequisites

`cargo build` will download and install the dependencies specified in
[`Cargo.toml`](Cargo.toml). In order to successfully test the application it
will need access to an SQS queue and if the queue contains any messages it will
attempt to access the DynamoDB table to retrieve records it is necessary to
have:

- An Amazon SQS queue URL
- A DynamoDB table name
- AWS credentials in the environment that can access both

### Test

```shell
cargo test
```

### Run

```
DRY_RUN="true" \
  AWS_REGION="us-east-1" \
  QUEUE_URL="https://sqs.us-east-1.amazonaws.com/<account_id>/<queue_name>" \
  cargo run
```

### Build

```shell
cargo build --release
```

## License

This Paw Extension is released under the [MIT License](LICENSE). Feel free to
fork, and modify!

Copyright © 2020 Bryan J Swift

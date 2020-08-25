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
[credential_chain_provider]: https://docs.rs/rusoto_credential/0.45.0/rusoto_credential/struct.ChainProvider.html

Other necessary configuration is provided by command line switches to the
`email_broker` program.

- `--region` defines the AWS region where the SQS queue and DynamoDB table are
  located. Specify "localstack" to use a "localhost" endpoint, otherwise
  [rusoto `Region`][region] is used to parse the region.
- `--queue-url` defines the SQS queue polled for messages.
- `--table-name` defines the name of the DynamoDB from which email messae data
  to send will be read.
- `--dry-run` when given the queue will only be polled a single time and no
  email information will be transmitted to the email sending service(s).

[region]: https://docs.rs/rusoto_core/0.45.0/rusoto_core/enum.Region.html

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

```shell
cargo run --bin email_broker -- \
  --dry-run \
  --region="<region>" \
  --queue-url="https://sqs.<region>.amazonaws.com/<account_id>/<queue_name>" \
  --table-name="<table_name>"
```

### Build

```shell
cargo build --release
```

#### Build for Lambda

```shell
make email_lambda.zip

# Create a lambda
aws lambda create-function --function-name <function_name> \
  --handler doesnt.matter \
  --zip-file fileb://./email_lambda.zip \
  --runtime provided \
  --role arn:aws:iam::<account_number>:role/<role_name> \
  --environment Variables={RUST_BACKTRACE=1} \
  --tracing-config Mode=Active

# Update the lambda
aws lambda update-function-code --function-name <function_name> \
  --zip-file fileb://./email_lambda.zip
```

## License

This experiment is released under the [MIT License](LICENSE). Feel free to
fork, and modify!

Copyright © 2020 Bryan J Swift

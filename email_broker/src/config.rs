use rusoto_core::Region;
use structopt::StructOpt;

const LOCALSTACK_REGION: &str = "localstack";

/// Create a custom `Region` if the given name is "localstack" otherwise determine `Region` from
/// the given string.
fn parse_region(s: &str) -> Region {
    if s == LOCALSTACK_REGION {
        Region::Custom {
            name: LOCALSTACK_REGION.into(),
            endpoint: "http://localhost:4566".into(),
        }
    } else {
        s.parse().unwrap_or(Region::default())
    }
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "email_broker",
    about = "Transmit pending email ids in SQS with data stored in DynamoDB"
)]
pub struct Options {
    /// Do not transmit emails
    #[structopt(long)]
    pub dry_run: bool,
    /// URL of SQS Queue from which email message ids will be read
    #[structopt(short = "q", long)]
    pub queue_url: String,
    /// AWS Region in which services reside
    #[structopt(short = "r", long, parse(from_str = parse_region))]
    pub region: Region,
    /// DynamoDB table from which email data will be read.
    #[structopt(short = "t", long)]
    pub table_name: String,
}

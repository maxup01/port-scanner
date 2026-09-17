use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    addr: String,
}

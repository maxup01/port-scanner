use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    #[arg(required = true, short, long)]
    address: String,

    #[arg(required = false, short, long = "fport")]
    first_port: Option<usize>,

    #[arg(required = false, short, long = "lport")]
    last_port: Option<usize>,
}

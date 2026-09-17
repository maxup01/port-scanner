use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    #[arg(required = true, short, long)]
    address: String,

    #[arg(required = false, short, long = "fport", default_value_t = 0)]
    first_port: usize,

    #[arg(required = false, short, long = "lport", default_value_t = 65535)]
    last_port: usize,
}

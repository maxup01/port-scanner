use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    #[arg(required = true, short, long)]
    pub address: String,

    #[arg(required = false, short, long = "fport", default_value_t = 0)]
    pub first_port: usize,

    #[arg(required = false, short, long = "lport", default_value_t = 65535)]
    pub last_port: usize,
}

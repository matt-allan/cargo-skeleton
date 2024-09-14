use anyhow::Result;
use cargo_skeleton::cli;
use clap::Parser;

fn main() -> Result<()> {
    env_logger::init();

    let args = cli::Cli::parse();

    cli::run(args)
}

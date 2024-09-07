use anyhow::{Context, Result};
use cargo_skeleton::{build, BuildOptions, DEFAULT_OUT_PATH};
use clap::{Arg, Command};
use camino::Utf8PathBuf;
//
fn cli() -> Command {
    // For an explanation of the expected command structure, see:
    // https://doc.rust-lang.org/cargo/reference/external-tools.html#custom-subcommands
    Command::new("cargo-skeleton")
        .about("Generate Cargo project skeletons")
        .subcommand_required(true)
        .subcommand(
            Command::new("skeleton")
                .about("Generate a bare bones skeleton from an existing Cargo project")
                .arg(
                    Arg::new("PATH")
                    .long("manifest-path")
                    .value_parser(clap::value_parser!(Utf8PathBuf))
                    .help("Path to Cargo.toml")
                    .required(false)
                )
                .arg(
                    Arg::new("OUT")
                    .short('o')
                    .long("out")
                    .value_parser(clap::value_parser!(Utf8PathBuf))
                    .default_value(DEFAULT_OUT_PATH)
                    .help("Path to write output to")
                    .required(false)
                ),
        )
}
fn main() -> Result<()> {
    env_logger::init();

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("skeleton", sub_matches)) => {
            let manifest_path = sub_matches.get_one::<Utf8PathBuf>("PATH").cloned();
            let out_path = sub_matches.get_one::<Utf8PathBuf>("OUT").cloned();

            let opts = BuildOptions {
                manifest_path,
                out_path,
            };

            build(opts).context("building skeleton")?;
        },
        _ => unreachable!(),
    }
    Ok(())
}

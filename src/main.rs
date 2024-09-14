use anyhow::{Context, Result};
use cargo_skeleton::{build_skeleton_package, create_skeleton, unpack_skeleton_archive, BuildOptions, CreateOptions, UnpackOptions, DEFAULT_OUT_PATH};
use clap::{Arg, Command};
use camino::Utf8PathBuf;

fn cli() -> Command {
    // For an explanation of the expected command structure, see:
    // https://doc.rust-lang.org/cargo/reference/external-tools.html#custom-subcommands
    Command::new("cargo-skeleton")
        .about("Generate Cargo project skeletons")
        .subcommand_required(true)
        .subcommand(
            Command::new("skeleton")
                .subcommand_required(true)
                .subcommand(
                    Command::new("create")
                    .about("Generate a skeleton archive")
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
                .subcommand(
                    Command::new("unpack")
                    .about("Unpack a skeleton archive")
                    .arg(
                        Arg::new("FILE")
                        .long("archive-file")
                        .value_parser(clap::value_parser!(Utf8PathBuf))
                        .default_value(DEFAULT_OUT_PATH)
                        .help("Path to skeleton archive")
                        .required(false)
                    )
                    .arg(
                        Arg::new("PATH")
                        .value_parser(clap::value_parser!(Utf8PathBuf))
                        .help("Destination path")
                        .required(false)
                    )
                )
                .subcommand(
                    Command::new("build")
                    .about("Build a skeleton project")
                    .arg(
                        // TODO: re-use this arg across commands
                        Arg::new("PATH")
                        .long("manifest-path")
                        .value_parser(clap::value_parser!(Utf8PathBuf))
                        .help("Path to Cargo.toml")
                        .required(false)
                    )
                    // TODO: pass everything after `--` to Cargo build
                )
        )
}
fn main() -> Result<()> {
    env_logger::init();

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("skeleton", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("create", sub_matches)) => {
                    let manifest_path = sub_matches.get_one::<Utf8PathBuf>("PATH").cloned();
                    let out_path = sub_matches.get_one::<Utf8PathBuf>("OUT").cloned();

                    let opts = CreateOptions {
                        manifest_path,
                        out_path,
                    };

                    create_skeleton(opts)
                        .context("building skeleton")?;

                    // TODO: print output
                },
                Some(("unpack", sub_matches)) => {
                    let archive_path = sub_matches.get_one::<Utf8PathBuf>("FILE").cloned();
                    let dest_path = sub_matches.get_one::<Utf8PathBuf>("PATH").cloned();

                    let opts = UnpackOptions {
                        archive_path,
                        dest_path,
                    };

                    unpack_skeleton_archive(opts)
                        .context("unpacking skeleton archive")?;

                    // TODO: print output
                },
                Some(("build", sub_matches)) => {
                    let manifest_path = sub_matches.get_one::<Utf8PathBuf>("PATH").cloned();

                    let opts = BuildOptions {
                        manifest_path,
                        // TODO: get from opts / metadata
                        packages: vec!["cargo-skeleton".into()],
                    };

                    build_skeleton_package(opts)
                        .context("building skeleton packages")?;
                },
                _ => unreachable!(),
            }
        },
        _ => unreachable!(),
    }
    Ok(())
}

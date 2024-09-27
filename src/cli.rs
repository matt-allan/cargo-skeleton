use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{Args, Parser};
use clap_cargo::style::{GOOD, CLAP_STYLING};

use crate::{
    build::{build_skeleton_package, BuildOptions},
    create::create_skeleton,
    unpack::{unpack_skeleton_archive, UnpackOptions},
};

#[derive(Debug, Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
#[command(version, about, long_about = None)]
pub enum Cli {
    #[command(subcommand)]
    Skeleton(SkeletonCommand),
}

#[derive(Debug, Parser)]
pub enum SkeletonCommand {
    Create(CreateArgs),
    Unpack(UnpackArgs),
    Build(BuildArgs),
}

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
pub struct CreateArgs {
    #[clap(flatten)]
    manifest: clap_cargo::Manifest,

    #[clap(flatten)]
    features: clap_cargo::Features,

    /// Path to write the skeleton archive to
    #[arg(long, default_value_t = Utf8PathBuf::from("skeleton.tar"))]
    out_path: Utf8PathBuf,
}

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
pub struct UnpackArgs {
    /// Path to the skeleton archive
    #[arg(long, default_value_t = Utf8PathBuf::from("skeleton.tar"))]
    archive_path: Utf8PathBuf,

    /// Output path for the archive contents
    #[arg(long, default_value_t = Utf8PathBuf::from("."))]
    out_path: Utf8PathBuf,
}

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
pub struct BuildArgs {
    #[clap(flatten)]
    manifest: clap_cargo::Manifest,

    #[clap(flatten)]
    workspace: clap_cargo::Workspace,

    /// Additional cargo build arguments
    #[arg(last = true)]
    args: Vec<String>,
}

pub fn run(args: Cli) -> Result<()> {
    let cmd = match args {
        Cli::Skeleton(cmd) => cmd,
    };

    match cmd {
        SkeletonCommand::Create(args) => {
            let metadata = {
                let mut metadata = args.manifest.metadata();

                args.features.forward_metadata(&mut metadata);

                metadata.exec().context("executing cargo metadata")?
            };

            println!("{GOOD}Creating{GOOD:#} {}", args.out_path);
            create_skeleton(metadata, args.out_path).context("building skeleton")?;
            println!("{GOOD}Finished{GOOD:#}");
        }
        SkeletonCommand::Unpack(args) => {
            let opts = UnpackOptions {
                archive_path: Some(args.archive_path.clone()),
                dest_path: Some(args.out_path),
            };

            println!("{GOOD}Unpacking{GOOD:#} {}", args.archive_path);
            unpack_skeleton_archive(opts).context("unpacking skeleton archive")?;
            println!("{GOOD}Finished{GOOD:#}");
        }
        SkeletonCommand::Build(args) => {
            let opts = BuildOptions {
                manifest_path: args
                    .manifest
                    .manifest_path
                    .map(|p| p.to_owned().try_into().unwrap()),
                packages: args.workspace.package,
                exclude: args.workspace.exclude,
                all: args.workspace.all,
                args: args.args,
            };

            build_skeleton_package(opts).context("building skeleton packages")?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;

        Cli::command().debug_assert();
    }
}

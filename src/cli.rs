use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{Args, CommandFactory, Parser};
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
    /// Create and build skeleton packages
    #[command(subcommand)]
    Skeleton(SkeletonCommand),
}

#[derive(Debug, Parser)]
pub enum SkeletonCommand {
    /// Create a skeleton archive from a Cargo workspace
    ///
    /// A skeleton is a tar archive of a workspace, containing
    /// all files necessary to compile the workspace dependencies.
    /// Files that affect compilation are copied as-is, while
    /// targets are replaced with empty stub files.
    /// 
    /// The skeleton archive is written to `skeleton.tar` in
    /// the current directory by default. To change the path,
    /// use the `--out-path` option.
    /// 
    /// The workspace packages, dependencies, and targets are
    /// discovered using Cargo metadata. By default Cargo
    /// searches for the `Cargo.toml` file in the current
    /// directory and any parent directories. To specify a
    /// different path, use the `--manifest-path` option.
    /// 
    /// Package dependencies are resolved when the archive
    /// is created. The feature selection flags may be used
    /// to control which features are enabled when Cargo
    /// resolves the workspace dependencies. All of the flags
    /// used by Cargo are supported: `--features`,
    /// `--all-features`, and `--no-default-features`.
    Create(CreateArgs),
    /// Unpack a skeleton archive
    /// 
    /// Unpacks the skeleton archive in the the given
    /// destination path.
    /// 
    /// If `--archive-path` is not specified, the command will
    /// look for a `skeleton.tar` in the current directory.
    /// 
    /// The archive is unpacked in the current directory
    /// unless `--dest-path` is specified. The archive is
    /// not deleted.
    /// 
    /// If the destination path contains a `Cargo.toml` and
    /// does not contain a `Skeleton.lock`, it is assumed to
    /// be an existing Cargo project. To prevent overwriting
    /// existing files, unpacking will fail.
    Unpack(UnpackArgs),
    /// Compile a skeleton package's dependencies
    /// 
    /// TODO(MJA): LONG HELP
    Build(BuildArgs),
    /// Generate man pages
    #[command(hide = true)]
    Mangen(MangenArgs),
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

#[derive(Debug, Args)]
#[command(hide = true)]
pub struct MangenArgs {
    /// Output path for the archive contents
    #[arg(long, default_value_t = Utf8PathBuf::from("."))]
    out_path: Utf8PathBuf,
}

pub fn run(cli: Cli) -> Result<()> {
    let cmd = match cli {
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
        },
        SkeletonCommand::Mangen(args) => {
            clap_mangen::generate_to(Cli::command(), args.out_path)
                .context("generating man pages")?;
        },
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

mod lockfile;

use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use cargo_metadata::{CargoOpt, Metadata, MetadataCommand, Package, Target};
use itertools::Itertools;
use lockfile::{Lockfile, Package as LockPackage, PackageId as LockPackageId, LOCKFILE_NAME};
use log::*;
use std::{env, fs::File, io::Write};

pub const DEFAULT_OUT_PATH: &str = "skeleton.tar";

const LIB_STUB: &str = r#"
// This file is automatically @generated by Cargo Skeleton.
// It is not intended for manual editing.
compile_error!("Attempted to compile skeleton file {}", file!());

"#;

const BIN_STUB: &str = r#"
// This file is automatically @generated by Cargo Skeleton.
// It is not intended for manual editing.
compile_error!("Attempted to compile skeleton file {}", file!());

fn main() {}

"#;

#[derive(Debug, Default)]
pub struct BuildOptions {
    pub manifest_path: Option<Utf8PathBuf>,
    pub out_path: Option<Utf8PathBuf>,
}

struct Builder<W: Write> {
    root_path: Utf8PathBuf,
    ar: tar::Builder<W>,
}

impl<W: Write> Builder<W> {
    pub fn new(root_path: impl Into<Utf8PathBuf>, out: W) -> Self {
        let root_path = root_path.into();
        let ar = tar::Builder::new(out);

        Self { root_path, ar }
    }

    pub fn build(mut self, metadata: &Metadata) -> Result<()> {
        env::set_current_dir(&metadata.workspace_root)
            .context("changing current directory to workspace root")?;

        self.add_root_cargo_files(metadata)?;

        let mut lockfile = Lockfile::default();

        let packages: Vec<&Package> = metadata
            .packages
            .iter()
            .filter(|pkg| pkg.source == None)
            .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
            .collect();

        for package in packages {
            self.add_package(package)?;

            add_lock_entry(metadata, package, &mut lockfile)
                .context("adding package lock entry")?;
        }

        self.add_lockfile(&mut lockfile)?;

        self.ar.into_inner().context("building tar archive")?;

        Ok(())
    }

    pub fn add_root_cargo_files(&mut self, metadata: &Metadata) -> Result<()> {
        if metadata.root_package().is_none() {
            self.ar
                .append_path("Cargo.toml")
                .context("adding root manifest to archive")?;
        }

        self.ar
            .append_path("Cargo.lock")
            .context("adding root Cargo.lock to archive")?;

        // TODO add these too:
        // - .cargo/config(.toml)?
        // - rust-toolchain(.toml)?

        Ok(())
    }

    pub fn add_package(&mut self, package: &Package) -> Result<()> {
        let manifest_path = self
            .make_relative(&package.manifest_path)
            .context("resolving package manifest path")?;

        self.add_manifest(&manifest_path)
            .context("adding package manifest")?;

        // TODO: add these files too:
        // - build.rs
        // - .cargo/config(.toml)? (at root but may be in a package too)
        // Because build.rs could reference anything, we probably should allow
        // using the manifest `metadata` field to reference arbitrary files that
        // should be included too.

        let targets: Vec<&Target> = package
            .targets
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
            .collect();

        for target in targets {
            self.add_target(&target)
                .context("adding package target to archive")?;
        }

        Ok(())
    }

    fn add_manifest(&mut self, path: &Utf8PathBuf) -> Result<()> {
        self.ar
            .append_path(path)
            .context("adding manifest to archive")?;

        Ok(())
    }

    fn add_lockfile(&mut self, lockfile: &mut Lockfile) -> Result<()> {
        let path: Utf8PathBuf = LOCKFILE_NAME.into();

        let data = lockfile.to_string();

        // TODO: dedupe this code
        let mut header = tar::Header::new_gnu();
        header.set_mtime(0);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_size(data.len().try_into().expect("no overflow"));
        header.set_cksum();

        self.ar
            .append_data(&mut header, path, data.as_bytes())
            .context("adding lockfile to archive")?;

        Ok(())
    }

    fn add_target(&mut self, target: &Target) -> Result<()> {
        let path = self
            .make_relative(&target.src_path)
            .context("resolving target path")?;

        let data =
            if target.is_bin() || target.is_bench() || target.is_test() || target.is_example() {
                BIN_STUB
            } else {
                LIB_STUB
            };

        let mut header = tar::Header::new_gnu();
        header.set_mtime(0);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_size(data.len().try_into().expect("no overflow"));
        header.set_cksum();

        self.ar
            .append_data(&mut header, path, data.as_bytes())
            .context("adding target stub to archive")?;

        Ok(())
    }

    fn make_relative(&self, path: &Utf8PathBuf) -> Result<Utf8PathBuf> {
        if !path.starts_with(&self.root_path) {
            bail!("Path outside of workspace root");
        }

        Ok(path
            .strip_prefix(&self.root_path)
            .expect("path inside workspace")
            .to_owned())
    }
}

pub fn build(opts: BuildOptions) -> Result<()> {
    let mut cmd = MetadataCommand::new();

    if let Some(manifest_path) = opts.manifest_path {
        cmd.manifest_path(manifest_path);
    }

    let metadata = cmd
        .features(CargoOpt::AllFeatures)
        .exec()
        .context("running cargo metadata")?;

    info!("Using workspace root: {}", metadata.workspace_root);

    let out_path = opts
        .out_path
        .unwrap_or_else(|| DEFAULT_OUT_PATH.try_into().expect("valid path"));

    info!("Writing to {}", out_path);

    let file = File::create(out_path).context("opening out file")?;

    let builder = Builder::new(&metadata.workspace_root, file);

    builder.build(&metadata)?;

    Ok(())
}

fn add_lock_entry(metadata: &Metadata, package: &Package, lockfile: &mut Lockfile) -> Result<()> {
    let deps = metadata
        .resolve.as_ref()
        .expect("resolve should be set unless --no-deps is specified")
        .nodes.iter()
        .find(|node| node.id == package.id)
        .ok_or_else(|| anyhow!("Missing package resolution for {}", package.id))?
        .dependencies.iter()
        .map(|dep| dep.to_string())
        .map_into::<LockPackageId>()
        .collect();

    let lock_pkg = LockPackage {
        name: package.name.clone(),
        id: package.id.to_string().into(),
        dependencies: deps,
    };

    lockfile.packages.push(lock_pkg);

    Ok(())
}
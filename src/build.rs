use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use std::{collections::HashSet, env, process::Command};
use log::*;

use crate::package::PackageId;
use crate::workspace::Workspace;

#[derive(Debug, Default)]
pub struct BuildOptions {
    pub manifest_path: Option<Utf8PathBuf>,

    pub packages: Vec<String>,
}

pub fn build_skeleton_package(opts: BuildOptions) -> Result<()> {
    let workspace_root: Utf8PathBuf = opts
        .manifest_path
        .and_then(|p| p.parent().map(|p| p.to_owned()))
        .unwrap_or_else(|| {
            env::current_dir()
                .expect("getting current dir")
                .try_into()
                .expect("current path should be utf-8")
        });

    let mut workspace = Workspace::new(workspace_root);

    workspace.load_lockfile()?;

    let build_ids: HashSet<&PackageId> = opts.packages
        .iter()
        .map(|spec| -> Result<&PackageId> {
            workspace.package_id(spec)
                .ok_or_else(|| anyhow!("package ID specification `{}` did not match any packages", &spec))
        })
        .collect::<Result<Vec<&PackageId>>>()?
        .into_iter()
        .flat_map(|id| workspace
            .get_package(id)
            .expect("present if ID was found")
            .dependencies
            .iter()
            .filter(|dep| workspace.is_member(dep))
            .chain(vec![id])
        )
        .collect::<HashSet<&PackageId>>();

    if build_ids.is_empty() {
        bail!("No packages to build");
    }

    let cargo = std::env::var("CARGO").unwrap_or("cargo".into());

    for pkg_id in build_ids.iter() {
        let pkg = workspace.get_package(pkg_id).expect("present if ID was found");  

        info!("Building package dependencies: {}", pkg.name);

        let pkg_args: Vec<&str> = pkg
            .dependencies.iter()
            .filter(|id| !workspace.is_member(id))
            .flat_map(|id| vec!["-p", id.as_str()])
            .collect();

        debug!("Running `cargo build {}`", pkg_args.join(" "));

        let mut child = Command::new(&cargo)
            .arg("build")
            .args(pkg_args)
            // TODO: cusom build args
            .arg("--release")
            .arg("--locked")
            .spawn()
            .context("executing `cargo build` command")?;

        let ecode = child.wait().expect("failed to wait on child");

        assert!(ecode.success());
    }

    Ok(())
}

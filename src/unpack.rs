use std::{env, fs::File};

use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;
use tar::Archive;

use crate::create::DEFAULT_OUT_PATH;

#[derive(Debug, Default)]
pub struct UnpackOptions {
    pub archive_path: Option<Utf8PathBuf>,
    pub dest_path: Option<Utf8PathBuf>,
}

pub fn unpack_skeleton_archive(opts: UnpackOptions) -> Result<()> {
    let archive_path = opts.archive_path.unwrap_or_else(|| DEFAULT_OUT_PATH.into());
    let dest_path = opts.dest_path.unwrap_or_else(|| {
        env::current_dir()
            .expect("getting current dir")
            .try_into()
            .expect("current path should be utf-8")
    });

    let file = File::open(archive_path).context("opening archive file")?;

    let mut ar = Archive::new(file);

    if dest_path.join("Cargo.toml").exists() &&
        !dest_path.join("Skeleton.lock").exists() {
            bail!("Attempted to unpack a skeleton archive into an existing workspace");
    }

    ar.unpack(dest_path).context("unpacking archive")?;

    Ok(())
}

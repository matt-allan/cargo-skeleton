use std::{collections::HashMap, ops::Index};

use anyhow::{anyhow, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::Metadata;
use itertools::Itertools;
use log::*;

use crate::{
    lockfile::{load_lockfile, Lockfile},
    package::{Package, PackageId},
};

/// A workspace of crates.
#[derive(Debug)]
pub struct Workspace {
    /// Path of the workspace's root directory
    workspace_root: Utf8PathBuf,

    /// Packages found within the workspace
    packages: HashMap<PackageId, Package>,
}

impl Workspace {
    pub fn new(workspace_root: Utf8PathBuf) -> Self {
        Self {
            workspace_root,
            packages: HashMap::new(),
        }
    }

    /// Returns the root path of the workspace.
    pub fn root(&self) -> &Utf8Path {
        &self.workspace_root
    }

    /// Load the workspace packages from a lockfile.
    pub fn load_lockfile(&mut self) -> Result<()> {
        let lockfile = load_lockfile(self.root())?;

        for package in lockfile.packages.into_iter() {
            self.add_package(package);
        }

        Ok(())
    }

    /// Load packages from workspace metadata.
    pub fn load_metadata(&mut self, metadata: &Metadata) -> Result<()> {
        let packages: Result<Vec<Package>> = metadata
            .packages
            .iter()
            .filter(|pkg| pkg.source == None)
            .filter(|pkg| {
                if !pkg.manifest_path.starts_with(self.root()) {
                    info!("Ignoring local package {} outside of root", pkg.name);
                    return false;
                }
                true
            })
            .sorted_by(|a, b| Ord::cmp(&a.id, &b.id))
            .map(|package| -> Result<Package> {
                let mut package = Package::from(package);

                package.load_metadata_dependencies(metadata)?;

                Ok(package)
            })
            .collect();

        for package in packages?.into_iter() {
            self.add_package(package);
        }

        Ok(())
    }

    /// Add a package to the workspace.
    pub fn add_package(&mut self, package: Package) {
        if self.packages.contains_key(&package.id) {
            return;
        }

        self.packages.insert(package.id.clone(), package);
    }

    /// Returns an iterator over all workspace packages.
    pub fn packages(&self) -> impl Iterator<Item = &Package> {
        self.packages.values()
    }

    /// Returns true if the workspace has a member with the given ID.
    pub fn is_member(&self, id: &PackageId) -> bool {
        self.packages.contains_key(id)
    }

    /// Get a package by ID.
    pub fn get_package(&self, id: &PackageId) -> Option<&Package> {
        self.packages.get(id)
    }

    /// Find package ids for a slice of specs.
    pub fn get_package_ids(&self, specs: &[impl AsRef<str>]) -> Result<Vec<&PackageId>> {
        specs
            .iter()
            .map(|spec| -> Result<&PackageId> {
                self.package_id(spec).ok_or_else(|| {
                    anyhow!(
                        "package ID specification `{}` did not match any packages",
                        spec.as_ref()
                    )
                })
            })
            .collect::<Result<Vec<&PackageId>>>()
    }

    /// Find the id of the package with the given spec.
    pub fn package_id(&self, spec: impl AsRef<str>) -> Option<&PackageId> {
        self.packages
            .values()
            .find(|pkg| pkg.name == spec.as_ref() || pkg.id.as_str() == spec.as_ref())
            .map(|pkg| &pkg.id)
    }

    /// Create a lockfile from the workspace.
    pub fn into_lockfile(self) -> Lockfile {
        Lockfile {
            packages: self.packages.into_values().collect(),
        }
    }
}

impl<'a> Index<&'a PackageId> for Workspace {
    type Output = Package;

    fn index(&self, idx: &'a PackageId) -> &Package {
        self.packages
            .get(idx)
            .unwrap_or_else(|| panic!("No package with id {:?}", idx))
    }
}

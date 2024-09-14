use std::fmt::{self, Display};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use cargo_metadata::{DependencyKind, Metadata, Package as MetaPackage, PackageId as MetaPackageId};

/// Meta information for a local package and it's dependencies.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct Package {
    /// The name field as given in Cargo.toml
    pub name: String,
    /// An opaque identifier for the package
    pub id: PackageId,
    /// The IDs of the resolved direct dependencies of the package
    pub dependencies: Vec<PackageId>,
}

impl Package {
    pub fn load_metadata_dependencies(&mut self, metadata: &Metadata) -> Result<()> {
        let deps: Vec<PackageId> = metadata
            .resolve.as_ref()
            .ok_or_else(|| anyhow!("Metadata missing deps"))?
            .nodes.iter()
            .find(|node| &node.id.repr == self.id.as_str())
            .ok_or_else(|| anyhow!("Missing package resolution for {}", self.id))?
            .deps.iter()
            .filter(|dep| dep.dep_kinds
                .iter()
                .any(|kind| kind.kind == DependencyKind::Normal)
            )
            .map(|dep| dep.pkg.clone().into())
            .collect();

        self.dependencies = deps;

        Ok(())
    }
}

impl From<&MetaPackage> for Package {
    fn from(package: &MetaPackage) -> Self {
        Self {
            name: package.name.clone(),
            id: package.id.clone().into(),
            dependencies: vec![],
        }
    }
}

/// An opaque identifier for a package.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct PackageId(String);

impl PackageId {
    pub fn as_str(&self) -> &str {
        &self.0 
    }
}

impl Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for PackageId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for PackageId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<MetaPackageId> for PackageId {
    fn from(id: MetaPackageId) -> Self {
        Self(id.repr)
    }
}
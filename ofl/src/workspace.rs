use anyhow::{Context, Error};
use cargo_metadata::{Metadata, Package, PackageId};
use git2::{ObjectType, Repository, Signature};
use std::{collections::BTreeMap, path::Path};
use tracing::info;

pub struct Workspace {
    pub metadata: Metadata,
    pub ofl_metadata: Metadata,
    pub repo: Repository,
}

impl Workspace {
    pub fn get(project_root: impl AsRef<Path>) -> Result<Self, Error> {
        let project_root = project_root.as_ref();

        Ok(Self {
            metadata: metadata_for_directory(project_root)?,
            ofl_metadata: metadata_for_directory(project_root.join("ofl"))?,
            repo: Repository::open(project_root)?,
        })
    }

    pub fn local_members(&self) -> Vec<PackageId> {
        local_metadata_members_reverse_topo(&self.metadata)
    }

    pub fn ofl_members(&self) -> Vec<PackageId> {
        local_metadata_members_reverse_topo(&self.ofl_metadata)
    }

    /// Returns a list of all crates which express a dependency upon `id` in
    /// their manifest.
    pub fn local_dependents(&self, id: &PackageId) -> Vec<Package> {
        let name = &self.metadata[id].name;
        self.metadata
            .packages
            .iter()
            .filter(|p| p.dependencies.iter().any(|d| &d.name == name))
            .cloned()
            .collect()
    }

    pub fn get_member(&self, id: &PackageId) -> Option<&Package> {
        if self.metadata.workspace_members.iter().any(|m| m == id) {
            Some(&self.metadata[id])
        } else {
            None
        }
    }

    /// Adds a new tag to the workspace repository which references HEAD.
    pub fn tag_head(&self, name: &str, message: &str) -> Result<(), Error> {
        let head = self.repo.head()?.peel(ObjectType::Commit)?;
        let tagger = Signature::now("Adam Perry", "lol@anp.lol")?;
        let tag = self.repo.tag(name, &head, &tagger, message, false)?;
        info!({ %name, %message, %tag }, "created tag");
        Ok(())
    }
}

fn metadata_for_directory(dir: impl AsRef<Path>) -> Result<Metadata, Error> {
    cargo_metadata::MetadataCommand::new()
        .manifest_path(dir.as_ref().join("Cargo.toml"))
        .current_dir(dir.as_ref())
        .exec()
        .context("collecting workspace metadata")
}

fn local_metadata_members_reverse_topo(metadata: &Metadata) -> Vec<PackageId> {
    let mut id_by_name: BTreeMap<String, PackageId> = Default::default();
    let mut dep_names_by_name: BTreeMap<String, Vec<String>> = Default::default();
    for id in &metadata.workspace_members {
        let package = &metadata[id];
        let name = &package.name;
        id_by_name.insert(name.clone(), id.clone());

        for dep in &package.dependencies {
            dep_names_by_name.entry(name.clone()).or_default().push(dep.name.clone());
        }
    }

    let member_names = id_by_name.keys().cloned().collect::<Vec<_>>();
    let member_names = pathfinding::prelude::topological_sort(&member_names, |name| {
        let mut deps = vec![];
        if let Some(names) = dep_names_by_name.get(name) {
            deps.extend(names.iter().cloned());
        }
        deps
    })
    .unwrap();

    member_names.into_iter().rev().filter_map(|name| id_by_name.get(&name)).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_workspace() {
        let _ws = Workspace::get(concat!(env!("CARGO_MANIFEST_DIR"), "/..")).unwrap();
    }
}

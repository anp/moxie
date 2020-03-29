use cargo_metadata::{Metadata, PackageId};
use failure::{Error, ResultExt};
use std::{collections::BTreeMap, path::Path};

pub struct Workspace {
    pub metadata: Metadata,
    pub ofl_metadata: Metadata,
    pub rustfmt_toolchain: String,
}

impl Workspace {
    pub fn get(project_root: impl AsRef<Path>) -> Result<Self, Error> {
        let project_root = project_root.as_ref();

        let mut rustfmt_toolchain = "nightly".to_string();
        for dir in project_root.ancestors() {
            if let Ok(tc) = std::fs::read_to_string(dir.join("rustfmt-toolchain")) {
                rustfmt_toolchain = tc;
                break;
            }
        }

        Ok(Self {
            metadata: metadata_for_directory(project_root)?,
            ofl_metadata: metadata_for_directory(project_root.join("ofl"))?,
            rustfmt_toolchain,
        })
    }

    pub fn local_members(&self) -> Vec<PackageId> {
        local_metadata_members_reverse_topo(&self.metadata)
    }

    pub fn ofl_members(&self) -> Vec<PackageId> {
        local_metadata_members_reverse_topo(&self.ofl_metadata)
    }
}

fn metadata_for_directory(dir: impl AsRef<Path>) -> Result<Metadata, Error> {
    Ok(cargo_metadata::MetadataCommand::new()
        .manifest_path(dir.as_ref().join("Cargo.toml"))
        .current_dir(dir)
        .exec()
        .context("collecting workspace metadata")?)
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

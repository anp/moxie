use cargo_metadata::Package;
use dialoguer::{Confirm, Select};
use failure::Error;
use gumdrop::Options;
use semver::Version;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use toml_edit::Document;

#[derive(Debug, Options)]
pub struct Versions {}

impl Versions {
    pub fn run(self, project_root: PathBuf) -> Result<(), Error> {
        let workspace = crate::workspace::Workspace::get(project_root)?;

        let moxie_crates = workspace.local_members();

        // TODO prompt for a subset of crates to update

        let mut updates = Vec::new();
        let mut updated_manifests = BTreeMap::new();
        for id in moxie_crates {
            let moxie_crate = workspace.get_member(&id).unwrap();

            if let Some(registries) = &moxie_crate.publish {
                if registries.is_empty() {
                    continue;
                }
            }

            let new_version = prompt_for_new_version(&moxie_crate)?;
            let manifest_contents = std::fs::read_to_string(&moxie_crate.manifest_path)?;

            if new_version != moxie_crate.version {
                // TODO ensure downstream updates increment versions appropriately
                updates.push((moxie_crate, new_version, workspace.local_dependents(&id)));
            }

            let manifest: Document = manifest_contents.parse()?;
            updated_manifests.insert(id, manifest);
        }

        let mut pending_updates = vec![String::from("crates to update:")];

        for (pkg, new_version, dependents) in &updates {
            let mut also_to_update = String::from("\t\tdependents: ");
            for reverse_dep in dependents {
                also_to_update.push_str(&reverse_dep.name);
                also_to_update.push_str(", ");
            }
            pending_updates.push(format!(
                "\t{}: {} -> {}\n{}",
                pkg.name, pkg.version, new_version, also_to_update
            ));
        }

        println!("{}", pending_updates.join("\n"));

        failure::ensure!(Confirm::new().with_prompt("proceed?").interact()?);

        for (id, new_version, dependents) in updates {
            // TODO update the version in its manifest
            // TODO update the dep version for each depending-upon manifest
        }

        for (id, manifest) in updated_manifests {
            // TODO write back to disk
        }

        Ok(())
    }
}

fn prompt_for_new_version(krate: &Package) -> Result<Version, Error> {
    let set_prerelease = |mut v: Version| {
        v.pre = vec![semver::Identifier::AlphaNumeric(String::from("pre"))];
        v
    };

    let mut options = vec![krate.version.clone()];

    options.push({
        let mut patched = krate.version.clone();
        patched.increment_patch();
        set_prerelease(patched)
    });
    options.push({
        let mut minored = krate.version.clone();
        minored.increment_minor();
        set_prerelease(minored)
    });
    options.push({
        let mut majored = krate.version.clone();
        majored.increment_major();
        set_prerelease(majored)
    });
    options.push({
        let mut promoted = krate.version.clone();
        promoted.pre = vec![];
        promoted
    });

    let selection = Select::new()
        .items(&options)
        .default(0)
        .with_prompt(format!("new version for `{}` (default no change)", krate.name))
        .interact()?;

    Ok(options[selection].clone())
}

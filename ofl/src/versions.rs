use anyhow::Error;
use dialoguer::{Confirm, Select};
use gumdrop::Options;
use semver::Version;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};
use toml_edit::Document;
use tracing::*;

#[derive(Debug, Options)]
pub struct Versions {}

impl Versions {
    pub fn run(self, project_root: PathBuf) -> Result<(), Error> {
        let workspace = crate::workspace::Workspace::get(project_root)?;

        let mut moxie_crates = workspace
            .local_members()
            .into_iter()
            .map(|id| workspace.get_member(&id).unwrap())
            .collect::<Vec<_>>();
        moxie_crates.sort_by_key(|c| &c.name);

        // prompt the user for which crates' versions they want bumped
        let mut choose_updates = Select::new();
        for krate in &moxie_crates {
            choose_updates.item(&krate.name);
        }

        let to_update_idx = choose_updates.with_prompt("Select crates to update:").interact()?;
        let to_update = moxie_crates[to_update_idx].clone();

        let mut updates = Vec::new();
        let mut updated_manifests = moxie_crates
            .iter()
            .map(|moxie_crate| {
                let manifest_contents = std::fs::read_to_string(&moxie_crate.manifest_path)?;
                let manifest: Document = manifest_contents.parse()?;
                Ok((moxie_crate.name.clone(), (moxie_crate, manifest)))
            })
            .collect::<Result<BTreeMap<_, _>, Error>>()?;

        if let Some(registries) = &to_update.publish {
            if registries.is_empty() {
                info!("TODO figure out why this happens again?");
                return Ok(());
            }
        }

        let new_version = prompt_for_new_version(&to_update.name, &to_update.version)?;

        if new_version != to_update.version {
            updates.push((
                to_update.clone(),
                new_version,
                workspace.local_dependents(&to_update.id),
            ));
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

        anyhow::ensure!(
            Confirm::new().with_prompt("proceed?").interact()?,
            "user must agree to proceed"
        );

        let mut to_write = BTreeSet::new();
        for (package, new_version, dependents) in updates {
            let manifest = &mut updated_manifests.get_mut(&package.name).unwrap().1;

            let new_version = toml_edit::value(new_version.to_string());
            manifest["package"]["version"] = new_version.clone();
            to_write.insert(package.id.clone());

            for dependent in dependents {
                let manifest = &mut updated_manifests.get_mut(&dependent.name).unwrap().1;

                if update_dependency_version(
                    manifest,
                    &package.name,
                    new_version.as_value().unwrap(),
                ) {
                    to_write.insert(dependent.id.clone());
                } else {
                    debug!({
                        crate = %dependent.name,
                        searched_for = %package.name,
                    }, "skipping, no dependency or version found");
                    continue;
                }
            }
        }

        for (package, manifest) in updated_manifests.values() {
            if !to_write.contains(&package.id) {
                debug!({ crate = %package.name }, "skipping, no file changes");
                continue;
            }

            info!({ crate = %package.name }, "writing updated manifest");
            std::fs::write(&package.manifest_path, manifest.to_string())?;
        }

        Ok(())
    }
}

/// returns true if an update was made, false if not
fn update_dependency_version<'doc>(
    manifest: &'doc mut Document,
    package_name: &str,
    new_version: &toml_edit::Value,
) -> bool {
    let dependencies = &mut manifest["dependencies"].as_table_mut().unwrap();

    if !dependencies.contains_key(package_name) {
        // it's not a direct dependency
        return false;
    }

    let dep_version: &mut toml_edit::Value =
        if let Some(dep) = dependencies[package_name].as_table_mut() {
            if dep.contains_key("version") {
                dep["version"].as_value_mut().unwrap()
            } else {
                // it's a table but it only specifies path or something else weird
                return false;
            }
        } else if let Some(dep) = dependencies[package_name].as_value_mut() {
            if let Some(inline) = dep.as_inline_table_mut() {
                if let Some(version) = inline.get_mut("version") {
                    version
                } else {
                    // it's an inline table but it only specifies path or something else weird
                    return false;
                }
            } else {
                dep
            }
        } else {
            // its not a table, not an inline table, and not a value...???
            return false;
        };

    *dep_version = new_version.clone();

    true
}

fn prompt_for_new_version(name: &str, version: &Version) -> Result<Version, Error> {
    let with_prerelease = |v: &Version| {
        let mut v = v.clone();
        v.pre = vec![semver::Identifier::AlphaNumeric(String::from("pre"))];
        v
    };

    let mut options = vec![version.clone()];

    if !version.pre.is_empty() {
        options.push({
            let mut promoted = version.clone();
            promoted.pre = vec![];
            promoted
        });
    }

    let mut patched = version.clone();
    patched.increment_patch();
    options.push(with_prerelease(&patched));
    options.push(patched);

    let mut minored = version.clone();
    minored.increment_minor();
    options.push(with_prerelease(&minored));
    options.push(minored);

    let mut majored = version.clone();
    majored.increment_major();
    options.push(with_prerelease(&majored));
    options.push(majored);

    let selection = Select::new()
        .items(&options)
        .default(0)
        .with_prompt(format!("new version for `{}` (default no change)", name))
        .interact()?;

    Ok(options[selection].clone())
}

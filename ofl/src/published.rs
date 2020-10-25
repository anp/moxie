use anyhow::{bail, Context, Error};
use cargo_metadata::{Package, PackageId};
use crates_index::Index;
use gumdrop::Options;
use std::path::PathBuf;
use tracing::*;

use crate::workspace::Workspace;

#[derive(Debug, Options)]
pub struct EnsurePublished {
    help: bool,
    /// Disables publishing to crates.io.
    #[options(no_short)]
    dry_run: bool,
}

impl EnsurePublished {
    pub fn run(self, root: PathBuf) -> Result<(), Error> {
        let workspace = Workspace::get(&root)?;
        info!(workspace = %root.display(), "identifying packages to publish");
        let to_publish = packages_to_publish(&workspace)?;
        for id in to_publish {
            let package = &workspace.metadata[&id];
            publish(package, self.dry_run)?;

            let tag = format!("{}-v{}", package.name, package.version);
            let message = format!("Published {}.", &tag);
            workspace.tag_head(&tag, &message)?;

            if !self.dry_run {
                info!("sleeping a bit");
                std::thread::sleep(std::time::Duration::from_secs(30));
            }
        }
        Ok(())
    }
}

fn packages_to_publish(workspace: &Workspace) -> Result<Vec<PackageId>, Error> {
    let index = Index::new_cargo_default();
    info!("updating cargo index");
    index.retrieve_or_update()?;

    let members = workspace.local_members();
    let mut to_publish_ids = vec![];
    let mut mismatched_checksums = vec![];

    for member in members {
        let package = &workspace.metadata[&member];
        let version_str = package.version.to_string();

        let manifest = std::fs::read_to_string(&package.manifest_path)?;
        if manifest.contains("publish = false") {
            debug!({ %package.name }, "skipping `publish = false`");
            continue;
        }

        if package.version.is_prerelease() {
            debug!({ %package.name, %package.version }, "skipping pre-release version");
            continue;
        }

        if let Some(krate) = index.crate_(&package.name) {
            if let Some(published) = krate.versions().iter().find(|v| v.version() == version_str) {
                info!(
                    { name = %published.name(), version = %published.version() },
                    "found already-published",
                );

                let current_checksum = workspace.member_checksum(&package.id)?;
                if &current_checksum != published.checksum() {
                    error!({ name = %published.name() }, "checksums don't match crates.io");
                    error!("consider running `cargo ofl versions` to update its version");
                    mismatched_checksums.push(package.name.clone());
                } else {
                    info!({ name = %published.name() }, "checksum matches crates.io");
                }

                continue;
            }
        }

        to_publish_ids.push(member);
    }

    if !mismatched_checksums.is_empty() {
        bail!(
            "{} crates' checksums were a mismatch to what's published without updated version \
             numbers.",
            mismatched_checksums.len()
        );
    }

    let to_publish =
        to_publish_ids.iter().map(|id| &workspace.metadata[id].name).collect::<Vec<_>>();

    info!("will publish: {:#?}", &to_publish);
    Ok(to_publish_ids)
}

fn publish(package: &Package, dry_run: bool) -> Result<(), Error> {
    info!({ %package.name, %package.version }, "publishing");

    let subcommand = if dry_run { "package" } else { "publish" };

    let output = std::process::Command::new("cargo")
        .arg(subcommand)
        .arg("--manifest-path")
        .arg(&package.manifest_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).context("output as utf8")?;
        let stdout = String::from_utf8(output.stdout).context("output as utf8")?;
        error!(
            "failed to package {}
stderr:
{}
stdout:
{}",
            package.manifest_path.display(),
            stderr,
            stdout,
        );
        bail!("cargo failure");
    }

    Ok(())
}

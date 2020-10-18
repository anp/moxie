use anyhow::{bail, Context, Error};
use cargo_metadata::{Package, PackageId};
use crates_io_api as crates;
use gumdrop::Options;
use semver::Version;
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
    let members = workspace.local_members();
    let mut to_publish_ids = vec![];

    for member in members {
        let package = &workspace.metadata[&member];

        let manifest = std::fs::read_to_string(&package.manifest_path)?;
        if manifest.contains("publish = false") {
            info!({ %package.name }, "skipping `publish = false`");
            continue;
        }

        if package.version.is_prerelease()
            || crates_io_has(&package.name, &package.version).unwrap_or(false)
        {
            info!({ %package.name, %package.version }, "skipping");
        } else {
            to_publish_ids.push(member);
        }
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

fn crates_io_has(name: &str, version: &Version) -> Result<bool, Error> {
    info!({ %name, %version }, "checking crates.io for");

    let client = crates::SyncClient::new("ofl", std::time::Duration::from_secs(1))?;
    let krate = client.full_crate(name, true /* all_versions */)?;
    let versions = &krate.versions;

    let current_version_str = version.to_string();

    Ok(versions.iter().map(|v| &v.num).any(|v| v == &current_version_str))
}

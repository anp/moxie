use {
    cargo_metadata::{Metadata, Package, PackageId},
    crates_io_api as crates,
    failure::{bail, Error, ResultExt},
    gumdrop::Options,
    semver::Version,
    std::collections::BTreeMap,
    tracing::*,
};

#[derive(Debug, Options)]
pub struct EnsurePublished {
    help: bool,
    /// Disables publishing to crates.io.
    #[options(no_short)]
    dry_run: bool,
}

impl EnsurePublished {
    pub fn run(self, metadata: Metadata) -> Result<(), Error> {
        let to_publish = packages_to_publish(&metadata)?;
        for id in to_publish {
            let package = &metadata[&id];
            publish(package, self.dry_run)?;
            info!("sleeping a bit");
            std::thread::sleep(std::time::Duration::from_secs(30));
        }
        Ok(())
    }
}

fn packages_to_publish(metadata: &Metadata) -> Result<Vec<PackageId>, Error> {
    let members = workspace_members_reverse_topo_sorted(metadata);
    let mut to_publish_ids = vec![];

    for member in members {
        let package = &metadata[&member];

        let manifest = std::fs::read_to_string(&package.manifest_path)?;
        if manifest.contains("publish = false") {
            info!({ %package.name }, "skipping `publish = false`");
            continue;
        }

        if package.version.to_string().ends_with("-pre")
            || crates_io_has(&package.name, &package.version).unwrap_or(true)
        {
            info!({ %package.name, %package.version }, "skipping");
        } else {
            to_publish_ids.push(member);
        }
    }

    let to_publish = to_publish_ids
        .iter()
        .map(|id| &metadata[id].name)
        .collect::<Vec<_>>();

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

    // TODO tag this commit with a version string

    Ok(())
}

fn crates_io_has(name: &str, version: &Version) -> Result<bool, Error> {
    info!({ %name, %version }, "checking crates.io for");

    let client = crates::SyncClient::new();
    let krate = client.full_crate(name, true /* all_versions */)?;
    let versions = &krate.versions;

    let current_version_str = version.to_string();

    Ok(versions
        .iter()
        .map(|v| &v.num)
        .any(|v| v == &current_version_str))
}

fn workspace_members_reverse_topo_sorted(metadata: &Metadata) -> Vec<PackageId> {
    let mut id_by_name: BTreeMap<String, PackageId> = Default::default();
    let mut dep_names_by_name: BTreeMap<String, Vec<String>> = Default::default();
    for id in &metadata.workspace_members {
        let package = &metadata[id];
        let name = &package.name;
        id_by_name.insert(name.clone(), id.clone());

        for dep in &package.dependencies {
            dep_names_by_name
                .entry(name.clone())
                .or_default()
                .push(dep.name.clone());
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

    member_names
        .into_iter()
        .rev()
        .filter_map(|name| id_by_name.get(&name))
        .cloned()
        .collect()
}

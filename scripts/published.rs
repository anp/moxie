//! ```cargo
//! [package]
//! edition = "2018"
//!
//! [dependencies]
//! cargo_metadata = "0.8.1"
//! crates_io_api = "0.5"
//! failure = "0.1"
//! gumdrop = "0.6"
//! pathfinding = "1.1.12"
//! semver = "0.9"
//! tracing = { version = "0.1", features = [ "log" ] }
//! tracing-fmt = "0.0.1-alpha.3"
//! ```

use {
    cargo_metadata::{Metadata, Package, PackageId},
    crates_io_api as crates,
    failure::{bail, Error},
    gumdrop::Options,
    semver::Version,
    std::{collections::BTreeMap, path::Path},
    tracing::*,
    tracing_fmt::{filter::env::EnvFilter, FmtSubscriber},
};

#[derive(Debug, Options)]
struct Config {
    help: bool,
    /// Disables publishing to crates.io.
    #[options(no_short)]
    dry_run: bool,
}

fn inputs() -> Result<(Config, Metadata), Error> {
    let scripts_path = std::env::var("CARGO_SCRIPT_BASE_PATH").unwrap();
    let root_path = Path::new(&scripts_path).parent().unwrap().to_path_buf();
    let config = Config::parse_args_default_or_exit();

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(root_path.join("Cargo.toml"))
        .current_dir(root_path)
        .exec()?;

    Ok((config, metadata))
}

fn main() -> Result<(), Error> {
    const RUST_LOG: &str = "debug";

    tracing::subscriber::with_default(
        FmtSubscriber::builder()
            .with_filter(EnvFilter::new(RUST_LOG))
            .finish(),
        || {
            debug!("logging init'd");
            ensure_published()
        },
    )
}

fn ensure_published() -> Result<(), Error> {
    let (config, metadata) = inputs()?;
    if config.dry_run {
        info!("dry run beginning");
    } else {
        warn!("LIVE");
    }

    let members = workspace_members_reverse_topo_sorted(&metadata);

    let mut pre_release_ids = vec![];
    let mut release_published_ids = vec![];
    let mut to_publish_ids = vec![];

    for member in members {
        let package = &metadata[&member];

        if package.version.to_string().ends_with("-pre") {
            pre_release_ids.push(member);
        } else if crates_io_has(&package.name, &package.version)? {
            release_published_ids.push(member);
        } else {
            to_publish_ids.push(member);
        }
    }

    let ids_to_names =
        |ids: &[PackageId]| ids.iter().map(|id| &metadata[id].name).collect::<Vec<_>>();

    let pre_release = ids_to_names(&pre_release_ids);
    let release_published = ids_to_names(&release_published_ids);
    let to_publish = ids_to_names(&to_publish_ids);

    info!({ ?pre_release, ?release_published }, "skipping");

    info!("will publish: {:#?}", &to_publish);

    to_publish_ids
        .iter()
        .map(|id| &metadata[id])
        .map(prepublish)
        .collect::<Result<Vec<()>, Error>>()?;

    if config.dry_run {
        info!("just kidding, it's a dry run");
    } else {
        warn!("PUBLISHING");
        bail!("TODO");
    }

    Ok(())
}

fn prepublish(package: &Package) -> Result<(), Error> {
    info!({ %package.name }, "prepublish verification");
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
        .find(|&v| v == &current_version_str)
        .is_some())
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

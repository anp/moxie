//! Tools for hacking on, contributing to, releasing, and otherwise maintaining the `moxie` project.
//!
//! ```cargo
//! [package]
//! name = "moxie-tool"
//! edition = "2018"
//!
//! [dependencies]
//! cargo_metadata = "0.8"
//! clap = "2"
//! dialoguer = "0.4"
//! failure = "0.1"
//! once_cell = "0.2"
//! pathfinding = "1"
//! salsa = "0.12"
//! structopt = "0.2"
//! ```

use {
    cargo_metadata::{Metadata, PackageId, Package},
    std::{
        path::{Path, PathBuf},
        sync::Arc,
    },
    structopt::StructOpt,
};

fn main() {
    let tool_rs_path = PathBuf::from(std::env::var("CARGO_SCRIPT_SCRIPT_PATH").unwrap());
    let repo_root = tool_rs_path.parent().unwrap().to_owned();

    let db = Db::init(repo_root);

    Runner::from_args().main(&db)
}

/// Internal tools for the moxie project.
#[derive(Debug, StructOpt)]
struct Runner {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Prepare the release process for one or more crates in the workspace.
    #[structopt(name = "stage-release")]
    StageRelease,
}

impl Runner {
    fn main(self, db: &Db) {
        match self.cmd {
            Command::StageRelease => {
                let to_stage = select_workspace_package(db);
                stage_release(db, to_stage);
            }
        }
    }
}

fn select_workspace_package(db: &Db) -> PackageId {
    let options = db.workspace_packages();

    let mut dialogue = dialoguer::Select::new();

    for krate in db.workspace_packages().iter() {
        dialogue.item(&*db.pkg_name(krate.clone()));
    }

    let k_idx = dialogue.interact().unwrap();
    options[k_idx].clone()
}

fn stage_release(db: &Db, krate: PackageId) {
    // TODO check that local git is clean, in sync with remote

    let mut to_consider = Vec::clone(&*db.workspace_deps(krate.clone()));
    to_consider.push(krate);
    println!(
        "We'll consider these crates for release: {:#?}",
        &to_consider
    );

    println!("sorting workspace crates by dep order");
    let release_order =
        pathfinding::directed::topological_sort::topological_sort(&*to_consider, |id| {
            Vec::clone(&*db.workspace_deps(id.clone()))
        })
        .unwrap()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();

    let mut test_cmd = std::process::Command::new("cargo");
    test_cmd.arg("test");

    for pkg in &release_order {
        test_cmd.args(&["--package", &*db.pkg_name(pkg.clone())]);
    }

    println!("running tests for crates: {:?}", &test_cmd);
    test_cmd.status().unwrap();

    for pkg in &release_order {
        let manifest_path = db.pkg
        std::process::Command::new("cargo").arg("package").arg("--manifest-path").arg()
    }
    // TODO git commit
    // TODO git tag
    // TODO post-increment version numbers with -dev suffix
    // TODO git commit
    unimplemented!()
}

fn workspace_packages(db: &impl MoxieTool) -> Arc<Vec<PackageId>> {
    Arc::new(db.cargo_md().workspace_members.clone())
}

fn workspace_deps(db: &impl MoxieTool, pkg: PackageId) -> Arc<Vec<PackageId>> {
    let workspace_members = db.workspace_packages();
    let pkg = &db.cargo_md()[&pkg];
    let mut deps = vec![];

    for dep in &pkg.dependencies {
        let id = db.pkg_by_name(Arc::new(dep.name.clone())).unwrap();

        if workspace_members.contains(&id) {
            deps.push(id.clone());
            deps.extend(db.workspace_deps(id.clone()).iter().cloned());
        }
    }

    Arc::new(deps)
}

fn pkg_by_name(db: &impl MoxieTool, name: Arc<String>) -> Option<PackageId> {
    for pkg in &db.cargo_md().packages {
        if &pkg.name == &*name {
            return Some(pkg.id.clone());
        }
    }

    None
}

fn pkg_name(db: &impl MoxieTool, id: PackageId) -> Arc<String> {
    Arc::new(db.pkg(id).name.clone())
}

fn pkg(db: &impl MoxieTool, id: PackageId) -> Arc<Package> {
    Arc::new(db.cargo_md()[&id].clone())
}

#[salsa::query_group(MoxieToolStorage)]
trait MoxieTool: salsa::Database {
    #[salsa::input]
    fn cargo_md(&self) -> Arc<Metadata>;

    fn pkg(&self, id: PackageId) -> Arc<Package>;
    fn pkg_by_name(&self, name: Arc<String>) -> Option<PackageId>;
    fn pkg_name(&self, id: PackageId) -> Arc<String>;

    fn workspace_packages(&self) -> Arc<Vec<PackageId>>;
    fn workspace_deps(&self, pkg: PackageId) -> Arc<Vec<PackageId>>;
}

#[salsa::database(MoxieToolStorage)]
struct Db {
    runtime: salsa::Runtime<Db>,
}

impl Db {
    fn init(repo_root: impl AsRef<Path>) -> Self {
        let metadata = cargo_metadata::MetadataCommand::new()
            .current_dir(&repo_root)
            .exec()
            .unwrap();

        let mut slf = Self {
            runtime: Default::default(),
        };

        slf.set_cargo_md(Arc::new(metadata));

        slf
    }
}

impl salsa::Database for Db {
    fn salsa_runtime(&self) -> &salsa::Runtime<Db> {
        &self.runtime
    }
}

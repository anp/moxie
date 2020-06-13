use failure::{bail, Error, ResultExt};
use gumdrop::Options;
use std::{
    fs::{create_dir, File},
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
};
use tracing::*;

/// project code coverage
#[derive(Debug, Options)]
pub enum Coverage {
    /// collect code coverage by running a cargo command
    Collect(Collect),
    /// generate a code coverage report
    Report(Report),
}

impl Coverage {
    pub fn run(&self, project_root: impl AsRef<Path>) -> Result<(), Error> {
        match self {
            Coverage::Collect(opts) => opts.run(project_root),
            Coverage::Report(opts) => opts.run(project_root),
        }
    }
}

/// run cargo with environment variables to collect source code coverage from
/// tests
#[derive(Debug, Options)]
pub struct Collect {
    /// the command to pass to cargo
    #[options(free)]
    args: Vec<String>,
}

impl Collect {
    /// Run cargo with the `coverage` profile and cfg enabled.
    pub fn run(&self, _project_root: impl AsRef<Path>) -> Result<(), Error> {
        let mut command = Command::new("cargo");
        command
            .env("RUSTDOCFLAGS", "-Cpanic=abort")
            .env("RUSTFLAGS", "-Zprofile -Zpanic_abort_tests -Clink-dead-code")
            .args(&self.args)
            .arg("-Zunstable-options")
            // defined in .cargo/config.toml
            .arg("--profile")
            .arg("coverage");
        debug!({ ?command }, "running");

        let status = command.status().context("running cargo command")?;
        if !status.success() {
            error!({ ?status }, "cargo failed");
            bail!("cargo failed! {:?}", status);
        }

        Ok(())
    }
}

/// run grcov on the `target/coverage` directory, generating a report
#[derive(Debug, Options)]
pub struct Report {}

impl Report {
    pub fn run(&self, source_root: impl AsRef<Path>) -> Result<(), Error> {
        let source_root = source_root.as_ref().to_owned();
        let output = source_root.join("target").join("coverage");
        info!("parsing coverage for html report...");
        let results = parse_coverage(&source_root);
        info!("generating html report...");
        grcov::output_html(results, Some(&*output.join("html").to_string_lossy()), 4);

        info!("parsing coverage for lcov report...");
        let results = parse_coverage(&source_root);
        info!("generating lcov report...");
        grcov::output_lcov(results, Some(&*output.join("lcov.info").to_string_lossy()));
        Ok(())
    }
}

fn parse_coverage(source_root: impl AsRef<Path>) -> grcov::CovResultIter {
    let source_root = source_root.as_ref().to_owned();
    let paths = [source_root.join("target").join("coverage").to_string_lossy().to_string()];
    let guess_directory = false;
    let ignore_not_existing = true;
    let is_llvm = true;
    let branch_enabled = true;
    let path_mapping_file = "";
    let filter_option = None;
    let prefix_dir = source_root.clone();
    let mut to_ignore_dirs = vec![];

    let tmp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let tmp_path = tmp_dir.path().to_owned();
    assert!(tmp_path.exists());

    let result_map: Arc<grcov::SyncCovResultMap> = Arc::new(Mutex::new(
        rustc_hash::FxHashMap::with_capacity_and_hasher(20_000, Default::default()),
    ));
    let (sender, receiver) = crossbeam::channel::bounded(2);
    let path_mapping = Arc::new(Mutex::new(None));

    let producer = {
        let sender = sender.clone();
        let tmp_path = tmp_path.clone();
        let path_mapping_file = path_mapping_file.to_owned();
        let path_mapping = Arc::clone(&path_mapping);

        std::thread::spawn(move || {
            let producer_path_mapping_buf = grcov::producer(
                &tmp_path,
                &paths,
                &sender,
                filter_option.is_some() && filter_option.unwrap(),
                is_llvm,
            );

            let mut path_mapping = path_mapping.lock().unwrap();
            *path_mapping = if path_mapping_file != "" {
                let file = File::open(path_mapping_file).unwrap();
                Some(serde_json::from_reader(file).unwrap())
            } else if let Some(producer_path_mapping_buf) = producer_path_mapping_buf {
                Some(serde_json::from_slice(&producer_path_mapping_buf).unwrap())
            } else {
                None
            };
        })
    };

    let result_map2 = Arc::clone(&result_map);
    let working_dir = tmp_path.join(format!("{}", 0));
    let source_root2 = source_root.clone();

    let consumer = std::thread::spawn(move || {
        create_dir(&working_dir).expect("Failed to create working directory");
        grcov::consumer(
            &working_dir,
            &Some(source_root2),
            &result_map2,
            receiver,
            branch_enabled,
            guess_directory,
        );
    });
    producer.join().expect("producer exits cleanly");
    sender.send(None).unwrap();
    consumer.join().expect("consumer exits cleanly");

    let result_map_mutex = Arc::try_unwrap(result_map).unwrap();
    let result_map = result_map_mutex.into_inner().unwrap();

    let path_mapping_mutex = Arc::try_unwrap(path_mapping).unwrap();
    let path_mapping = path_mapping_mutex.into_inner().unwrap();

    grcov::rewrite_paths(
        result_map,
        path_mapping,
        Some(source_root),
        Some(prefix_dir),
        ignore_not_existing,
        &mut to_ignore_dirs,
        filter_option,
        grcov::FileFilter::new(None, None, None, None, None, None),
    )
}

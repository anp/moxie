use failure::{Error, SyncFailure};
use gumdrop::Options;
use mdbook::MDBook;
use std::path::{Path, PathBuf};
use tracing::*;

#[derive(Debug, Options)]
pub struct Website {
    help: bool,
    #[options(command)]
    op: Option<Operation>,
}

impl Website {
    pub fn run(self, root_path: PathBuf) -> Result<(), Error> {
        let operation = self.op.unwrap_or_else(|| Operation::Build(DistOpts::default(&root_path)));
        match operation {
            Operation::Build(opts) => opts.build_website_dist(&root_path),
        }
    }
}

#[derive(Debug, Options)]
enum Operation {
    Build(DistOpts),
}

#[derive(Debug, Options)]
struct DistOpts {
    help: bool,
    #[options(free, required)]
    output_dir: PathBuf,
}

impl DistOpts {
    fn default(root_path: &Path) -> Self {
        Self { help: false, output_dir: root_path.join("target").join("website") }
    }

    fn build_website_dist(self, root_path: &Path) -> Result<(), Error> {
        let md = MDBook::load(&root_path.join("book")).map_err(SyncFailure::new)?;
        md.build().map_err(SyncFailure::new)?;
        self.copy_to_target_dir(root_path)
    }

    fn copy_to_target_dir(self, root_path: &Path) -> Result<(), Error> {
        let _ = std::fs::remove_dir_all(&self.output_dir);
        std::fs::create_dir_all(&self.output_dir)?;
        let output_path = self.output_dir.canonicalize()?;

        let to_copy = self.files_to_copy(root_path, &output_path)?;
        info!({ num_files = to_copy.len() }, "discovered");

        for path in to_copy {
            let relative = path.strip_prefix(root_path)?;
            let rel_path = relative.display();
            debug!({ %rel_path }, "copying path");
            let destination = output_path.join(relative);
            std::fs::create_dir_all(destination.parent().unwrap())?;
            std::fs::copy(path, destination)?;
        }

        Ok(())
    }

    fn files_to_copy(&self, root_path: &Path, output_path: &Path) -> Result<Vec<PathBuf>, Error> {
        let skip_prefixes =
            vec![output_path.to_path_buf(), root_path.join(".vscode"), root_path.join("ofl")];

        let exts = vec!["css", "html", "ico", "js", "map", "png", "svg", "txt", "wasm", "woff"];

        let output = output_path.display();
        info!({ %output }, "cleaning");

        info!("discovering files to copy");
        let mut to_copy = vec![];
        'entries: for entry in walkdir::WalkDir::new(root_path) {
            let path = entry?.path().to_owned();

            match path.extension() {
                Some(ext) if exts.contains(&ext.to_str().unwrap()) => (),
                _ => continue,
            };

            for prefix in &skip_prefixes {
                if path.starts_with(prefix) {
                    continue 'entries;
                }
            }
            to_copy.push(path);
        }
        Ok(to_copy)
    }
}

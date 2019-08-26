use {failure::Error, gumdrop::Options, std::path::Path, tracing::*};

#[derive(Debug, Options)]
pub struct Website {
    help: bool,
}

impl Website {
    pub fn run(self, root_path: &Path) -> Result<(), Error> {
        let tools_path = root_path.join("ofl");

        let root_target_dir = root_path.join("target");
        let tools_target_dir = tools_path.join("target");
        let output_path = root_target_dir.join("website");
        let output = output_path.display();

        let skip_prefixes = vec![
            tools_path,
            tools_target_dir,
            root_target_dir,
            root_path.join(".vscode"),
        ];

        let exts = vec!["css", "html", "js", "png", "svg", "wasm"];

        info!({ %output }, "cleaning, copying files");
        let _ = std::fs::remove_dir_all(&output_path);
        std::fs::create_dir_all(&output_path)?;

        'entries: for entry in walkdir::WalkDir::new(root_path) {
            let path = entry?.path().to_owned();

            match path.extension() {
                Some(ext) if exts.contains(&ext.to_str().unwrap()) => (),
                _ => continue,
            };

            let relative = path.strip_prefix(root_path)?;
            let rel_path = relative.display();
            let destination = output_path.join(relative);

            for prefix in &skip_prefixes {
                if path.starts_with(prefix) {
                    continue 'entries;
                }
            }

            info!({ %rel_path }, "copying path");
            std::fs::create_dir_all(destination.parent().unwrap())?;
            std::fs::copy(path, destination)?;
        }

        Ok(())
    }
}

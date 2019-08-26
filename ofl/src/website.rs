use {
    failure::{Error},
    std::path::Path,
    tracing::*,
    tracing_fmt::{filter::env::EnvFilter, FmtSubscriber},
};

fn copy_website_files_to_dest() -> Result<(), Error> {
    let tools_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let root_path = tools_path
        .parent()
        .unwrap();

    let root_target_dir = root_path.join("target");
    let tools_target_dir = tools_path.join("target");
    let output_path = root_target_dir.join("website");
    let output = output_path.display();

    let skip_prefixes = vec![
        root_target_dir,
        root_path.join(".vscode"),
        tools_target_dir,
    ];

    let exts = vec![
        "css",
        "html",
        "js",
        "png",
        "svg",
        "wasm",
    ];

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

fn main() -> Result<(), Error> {
    const RUST_LOG: &str = "debug";

    tracing::subscriber::with_default(
        FmtSubscriber::builder()
            .with_filter(EnvFilter::new(RUST_LOG))
            .finish(),
        || {
            debug!("logging init'd");
            copy_website_files_to_dest()?;
            Ok(())
        },
    )
}

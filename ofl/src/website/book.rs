
use {failure::{Error, SyncFailure}, mdbook::MDBook, std::path::Path};

pub fn build_book(book_root: &Path, output_dir: &Path) -> Result<(), Error> {
    let mut md = MDBook::load(book_root).map_err(SyncFailure::new)?;
    md.build().map_err(SyncFailure::new)?;
    unimplemented!();
    Ok(())
}


use mdbook::MDBook;

fn build_book(book_root: &Path, output_dir: &Path) -> Result<(), Error> {
    let mut md = MDBook::load(book_root)?;
    md.build()?;
    unimplemented!()
    Ok(())
}

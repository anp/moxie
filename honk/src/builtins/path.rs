use starlark::values::{error::ValueError, list::List, none::NoneType, TypedValue, Value};
use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};

starlark_module! { globals =>
    honk_root() {
        // TODO handle error here
        Ok(Value::new(Path::new(std::env::current_dir().unwrap())))
    }

    path(p: String) {
        Ok(Value::new(Path::new(PathBuf::from(p))))
    }

    Path.exists(this: Path) {
        Ok(Value::new(this.exists()))
    }

    Path.parent(this: Path) {
        Ok(opt_typed_val(this.parent()))
    }

    Path.filename(this: Path) {
        Ok(opt_typed_val(this.filename()))
    }

    Path.join(this: Path, to_join: String) {
        Ok(Value::new(this.join(&to_join)))
    }

    Path.canonicalize(this: Path) {
        Ok(Value::new(this.canonicalize()))
    }

    Path.glob(this: Path, pattern: String) {
        Ok(Value::new(this.globs(&[pattern])))
    }

    Path.globs(this: Path, patterns: Vec<String>) {
        Ok(Value::new(this.globs(&patterns)))
    }
}

fn opt_typed_val(v: Option<impl TypedValue>) -> Value {
    if let Some(v) = v { Value::new(v) } else { Value::new(NoneType::None) }
}

#[derive(Clone, Debug, Hash)]
pub struct Path {
    inner: PathBuf,
}

impl Path {
    fn new(inner: PathBuf) -> Self {
        Self { inner }
    }

    fn exists(&self) -> bool {
        self.inner.exists()
    }

    fn parent(&self) -> Option<Self> {
        self.inner.parent().map(|p| Self { inner: p.to_path_buf() })
    }

    fn filename(&self) -> Option<String> {
        self.inner.file_name().map(|n| n.to_string_lossy().to_string())
    }

    fn join(&self, to_join: &str) -> Self {
        Self { inner: self.inner.join(to_join) }
    }

    fn canonicalize(&self) -> Self {
        // TODO handle errors here
        Self { inner: self.inner.canonicalize().unwrap() }
    }

    // TODO return a Set once exposed from starlark crate
    fn globs(&self, patterns: &[impl AsRef<str>]) -> List {
        let mut results = List::default();

        for pattern in patterns {
            let pattern = pattern.as_ref();
            // FIXME this might be a broken way to do globs "scoped" to a parent path?
            let full_pattern = self.inner.to_string_lossy() + "/" + pattern;

            // TODO glob against the vfs
            for entry in glob::glob(&full_pattern).expect("must pass a valid glob") {
                let matched = Path::new(entry.unwrap());
                results.push(Value::new(matched)).unwrap();
            }
        }

        results
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.display().fmt(f)
    }
}

impl TypedValue for Path {
    type Holder = starlark::values::Immutable<Self>;

    const TYPE: &'static str = "Path";

    fn values_for_descendant_check_and_freeze(&self) -> Box<dyn Iterator<Item = Value> + '_> {
        Box::new(std::iter::empty())
    }

    fn get_hash(&self) -> Result<u64, ValueError> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        Ok(hasher.finish())
    }

    fn to_repr_impl(&self, buf: &mut String) -> std::fmt::Result {
        use std::fmt::Write;
        write!(buf, "{}", self)
    }
}

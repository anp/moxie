use starlark::values::{error::ValueError, list::List, none::NoneType, TypedValue, Value};
use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};

starlark_module! { globals =>
    path(p: String) {
        Ok(Value::new(Path::new(PathBuf::from(p))))
    }

    Path.parent(this: Path) {
        Ok(opt_typed_val(this.parent()))
    }

    Path.filename(this: Path) {
        Ok(opt_typed_val(this.filename()))
    }

    Path.glob(this: Path, pattern: String) {
        Ok(Value::new(this.glob(&pattern)))
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

    fn parent(&self) -> Option<Self> {
        self.inner.parent().map(|p| Self { inner: p.to_path_buf() })
    }

    fn filename(&self) -> Option<String> {
        self.inner.file_name().map(|n| n.to_string_lossy().to_string())
    }

    // TODO return a Set once exposed from starlark crate
    fn glob(&self, pattern: &str) -> List {
        let mut results = List::default();
        // FIXME this might be a broken way to do globs "scoped" to a parent path?
        let full_pattern = self.inner.to_string_lossy() + "/" + pattern;

        // TODO glob against the vfs
        for entry in glob::glob(&full_pattern).expect("must pass a valid glob") {
            let matched = Path::new(entry.unwrap());
            results.push(Value::new(matched)).unwrap();
        }

        results
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
        buf.push_str(&self.inner.to_string_lossy());
        Ok(())
    }
}

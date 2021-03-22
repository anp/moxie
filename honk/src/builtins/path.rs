use starlark::{
    environment::GlobalsBuilder,
    starlark_immutable_value, starlark_type,
    values::{list::List, AllocValue, Heap, TypedValue, Value},
};
use starlark_module::starlark_module;
use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};

#[starlark_module]
pub fn register(builder: &mut GlobalsBuilder) {
    fn honk_root() -> HonkPath {
        Ok(honk_root_impl())
    }

    fn path(p: &str) -> HonkPath {
        Ok(HonkPath::new(p))
    }
}

fn honk_root_impl() -> HonkPath {
    // FIXME get this from the CLI arg
    HonkPath { inner: std::env::current_dir().unwrap() }
}

fn opt_typed_val<'h>(heap: &'h Heap, v: Option<impl AllocValue<'h>>) -> Value<'h> {
    if let Some(v) = v {
        heap.alloc(v)
    } else {
        Value::new_none()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HonkPath {
    inner: PathBuf,
}

impl HonkPath {
    fn new(inner: &str) -> Self {
        Self {
            inner: if inner.starts_with("//") {
                honk_root_impl().inner.join(inner.trim_start_matches("//"))
            } else {
                PathBuf::from(inner)
            },
        }
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
    fn globs<'h>(&self, heap: &'h Heap, patterns: &[impl AsRef<str>]) -> List<'h> {
        let mut results = List::default();

        for pattern in patterns {
            let pattern = pattern.as_ref();
            // FIXME this might be a broken way to do globs "scoped" to a parent path?
            let full_pattern = self.inner.to_string_lossy() + "/" + pattern;

            // TODO glob against the vfs
            for entry in glob::glob(&full_pattern).expect("must pass a valid glob") {
                results.push(heap.alloc(HonkPath { inner: entry.unwrap() }));
            }
        }

        results
    }
}

impl std::fmt::Display for HonkPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.display().fmt(f)
    }
}

#[starlark_module::starlark_module]
fn register_path_methods(globals: &mut GlobalsBuilder) {
    fn exists(this: RefHonkPath) -> bool {
        Ok(this.exists())
    }

    fn parent(this: RefHonkPath) -> Value<'v> {
        Ok(opt_typed_val(heap, this.parent()))
    }

    fn filename(this: RefHonkPath) -> Value<'v> {
        Ok(opt_typed_val(heap, this.filename()))
    }

    fn join(this: RefHonkPath, to_join: &str) -> HonkPath {
        Ok(this.join(&to_join))
    }

    fn canonicalize(this: RefHonkPath) -> HonkPath {
        Ok(this.canonicalize())
    }

    fn glob(this: RefHonkPath, pattern: &str) -> List<'v> {
        Ok(this.globs(heap, &[pattern]))
    }

    fn globs(this: RefHonkPath, patterns: Vec<&str>) -> List<'v> {
        Ok(this.globs(heap, &patterns))
    }
}

impl TypedValue<'_> for HonkPath {
    starlark_type!("path");
    declare_members!(register_path_methods);

    fn get_hash(&self) -> anyhow::Result<u64> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        Ok(hasher.finish())
    }

    fn collect_repr(&self, collector: &mut String) {
        use std::fmt::Write;
        write!(collector, "{}", self).expect("when can write! into a string even fail???");
    }
}

starlark_immutable_value!(pub HonkPath);

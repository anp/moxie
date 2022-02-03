use crate::{builtins::command::HonkCommand, EvaluatorExt};
use starlark::{
    environment::GlobalsBuilder,
    values::{none::NoneType, ARef, Value},
};
use std::collections::BTreeSet;

#[starlark_module::starlark_module]
pub fn register(globals: &mut GlobalsBuilder) {
    fn target(name: &str, command: ARef<HonkCommand>, deps: Option<Vec<Value<'_>>>) -> NoneType {
        let deps: DepSet = deps.into();
        ctx.revision().register_target(name, &*command, &deps);
        Ok(NoneType)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DepSet {
    inner: BTreeSet<String>,
}

impl<'a> From<Option<Vec<Value<'a>>>> for DepSet {
    fn from(v: Option<Vec<Value<'a>>>) -> Self {
        Self {
            inner: v
                .into_iter()
                .map(IntoIterator::into_iter)
                .flatten()
                .map(|a| a.to_str())
                .collect(),
        }
    }
}

impl<'a> IntoIterator for &'a DepSet {
    type Item = &'a String;
    type IntoIter = <&'a BTreeSet<String> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&self.inner).into_iter()
    }
}

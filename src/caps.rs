#[macro_export]
macro_rules! mox {
    ($scope:ident <- $props:expr) => {{
        $scope.compose_child($crate::scope!($scope.id()), $props);
    }};
}

#[macro_export]
macro_rules! state {
    ($scope:ident <- $init:expr) => {
        $scope.state($crate::callsite!($scope.id()), || $init)
    };
}

#[macro_export]
macro_rules! task {
    ($scope:ident <- $body:expr) => {
        $crate::task_fut!($scope <- async move { $body })
    };
}

#[macro_export]
macro_rules! task_fut (
    ($scope:ident <- $body:expr) => {
        $scope.task($crate::callsite!($scope.id()), ($body))
    };
);

/// A `Moniker` represents the coordinates of a code location in the render hierarchy.
///
/// The struct describes a location in the program specific to:
///
/// * a line and column of code,
/// * in a particular element function,
/// * as well as the moniker which resulted in that particular function's invocation
///
/// It can be derived at any point within any element as long as the parent/invoking/enclosing
/// moniker is available. We guarantee that it's always available in render lifecycle in other ways.
///
/// `Moniker`s are the tool underlying elements, state, context, etc. because they allow us to map
/// from a "pure" function back to a state location.
#[doc(hidden)]
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct Moniker(usize);

impl Moniker {
    #[doc(hidden)]
    #[inline]
    pub fn new(scope: ScopeId, callsite: &'static str) -> Self {
        Moniker(fxhash::hash(&(scope, callsite)))
    }
}

impl std::fmt::Debug for Moniker {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("m{:#x}", self.0))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! moniker {
    ($parent:expr) => {
        $crate::Moniker::new($parent, concat!(file!(), "@", line!(), ":", column!()))
    };
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScopeId(Moniker);

impl ScopeId {
    #[doc(hidden)]
    pub fn new(callsite: Moniker) -> Self {
        Self(callsite)
    }

    pub(crate) fn root() -> Self {
        Self(Moniker(fxhash::hash(&0)))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! scope {
    ($parent:expr) => {
        $crate::ScopeId::new($crate::moniker!($parent))
    };
}

#[doc(hidden)]
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct CallsiteId {
    scope: ScopeId,
    site: Moniker,
}

impl CallsiteId {
    #[doc(hidden)]
    pub fn new(scope: ScopeId, site: Moniker) -> Self {
        Self { scope, site }
    }
}

impl std::fmt::Debug for CallsiteId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("CallsiteId").field(&self.site).finish()
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! callsite {
    ($parent:expr) => {
        $crate::CallsiteId::new($parent, $crate::moniker!($parent))
    };
}

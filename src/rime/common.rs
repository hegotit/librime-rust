use std::path::PathBuf;
use std::sync::Arc;

// Call a pointer to member function on this
macro_rules! rime_this_call {
    ($this:expr, $f:ident) => {
        ($this.$f)()
    };
}

// Call a pointer to member function on this as a specific type
macro_rules! rime_this_call_as {
    ($this:expr, $T:ty, $f:ident) => {
        ($this as *const $T).$f()
    };
}

// Type definitions
pub type Unique<T> = Box<T>;
pub type Shared<T> = Arc<T>;

// Convert Shared to another type
pub fn as_shared<X, Y>(ptr: &Shared<Y>) -> Option<Shared<X>>
    where
        Y: ?Sized + 'static,
        X: ?Sized + 'static,
{
    ptr.clone().downcast::<X>().ok()
}

// Check if Shared is of a specific type
pub fn is_shared<X, Y>(ptr: &Shared<Y>) -> bool
    where
        Y: ?Sized + 'static,
        X: ?Sized + 'static,
{
    as_shared::<X, Y>(ptr).is_some()
}

// Create a new Shared instance
pub fn new_shared<T>(value: T) -> Shared<T> {
    Arc::new(value)
}

// Custom path structure
#[derive(Debug, Clone)]
pub struct PathWrapper {
    inner: PathBuf,
}

impl PathWrapper {
    pub fn new() -> Self {
        Self {
            inner: PathBuf::new(),
        }
    }

    pub fn from_str(path: &str) -> Self {
        Self {
            inner: PathBuf::from(path),
        }
    }

    // Operator overloading: Path concatenation
    pub fn join(&self, other: &Self) -> Self {
        Self {
            inner: self.inner.join(&other.inner),
        }
    }
}

// Operator overloading: Path concatenation
impl std::ops::Div for PathWrapper {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self.join(&rhs)
    }
}

// Custom logging output (only when logging is enabled)
#[cfg(feature = "logging")]
impl std::fmt::Display for PathWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.display())
    }
}
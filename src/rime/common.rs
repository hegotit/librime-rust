use std::ops::{Div, DivAssign};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct PathExt {
    pub(crate) path: PathBuf,
}

impl PathExt {
    pub(crate) fn new() -> Self {
        Self {
            path: PathBuf::new(),
        }
    }

    pub(crate) fn from<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub(crate) fn join<P: AsRef<Path>>(&self, other: P) -> Self {
        Self {
            path: self.path.join(other),
        }
    }

    pub(crate) fn exists(&self) -> bool {
        self.path.exists()
    }
}

impl AsRef<Path> for PathExt {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl<T> Div<T> for PathExt
where
    T: AsRef<Path>,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        self.join(rhs)
    }
}

impl<T> DivAssign<T> for PathExt
where
    T: AsRef<Path>,
{
    fn div_assign(&mut self, rhs: T) {
        self.path = self.path.join(rhs);
    }
}

// Custom logging output (only when logging is enabled)
#[cfg(feature = "logging")]
impl std::fmt::Display for PathExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

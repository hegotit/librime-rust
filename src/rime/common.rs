use std::fmt::Display;
use std::ops::{Deref, DerefMut, Div, DivAssign};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PathExt(pub(crate) PathBuf);

impl PathExt {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            0: path.as_ref().to_path_buf(),
        }
    }

    pub(crate) fn join<P>(&self, other: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            0: self.0.join(other),
        }
    }
}

impl Deref for PathExt {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PathExt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Path> for PathExt {
    fn as_ref(&self) -> &Path {
        &self.0
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
        self.0 = self.0.join(rhs);
    }
}

impl Display for PathExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

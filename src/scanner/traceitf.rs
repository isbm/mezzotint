use std::path::PathBuf;

/// Package dependency trace
pub trait PkgDepTrace {
    fn trace(&mut self, pkgname: String) -> Vec<String>;
    fn exclude(&mut self, pkgs: Vec<String>) -> &mut dyn PkgDepTrace;
}

pub trait PkgFileTrace {
    /// Return a package name, to which this file belongs to
    fn trace(&mut self, filename: PathBuf) -> Option<String>;
}

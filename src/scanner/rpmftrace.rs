use std::{collections::HashMap, path::PathBuf};

use super::traceitf::PkgFileTrace;

/// Trace what package a give file belongs to, using RPM package manager.
pub struct RpmPkgFileTrace {
    file_to_pkg: HashMap<PathBuf, String>,
}

impl RpmPkgFileTrace {
    pub fn new() -> Self {
        RpmPkgFileTrace { file_to_pkg: HashMap::default() }
    }
}

impl PkgFileTrace for RpmPkgFileTrace {
    fn trace(&mut self, filename: PathBuf) -> Option<String> {
        None
    }
}

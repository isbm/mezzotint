use std::{collections::HashSet, process::Command};

use super::traceitf::PkgDepTrace;

pub struct DebPackageTrace {
    data: HashSet<String>,
}

impl DebPackageTrace {
    pub fn new() -> Self {
        DebPackageTrace { data: HashSet::default() }
    }

    /// Get list of package dependencies for the first nearby level
    ///
    /// NOTE: currently it is quite bad way by hammering with apt,
    ///       but it works and is okey-ish for the time being.
    ///       This needs to be rewritten by slurping the entire /var/lib/dpkg/status
    ///       and the processing it at once.
    fn get_dependencies(&mut self, pkg: String, start: bool) -> Vec<String> {
        if start {
            self.data.clear();
        }

        let mut c = Command::new("apt");
        c.args(["depends", pkg.as_str()]);

        match c.output() {
            Ok(out) => {
                if let Ok(out) = String::from_utf8(out.stdout) {
                    for l in out.lines().map(|s| s.trim().to_string()).collect::<Vec<String>>() {
                        if l.to_lowercase().starts_with("depends:") {
                            let l = l.split(" ").into_iter().collect::<Vec<&str>>();
                            if l.len() > 2 {
                                let pkgname = l[1].to_string();
                                if !self.data.contains(&pkgname) {
                                    self.data.insert(pkgname.to_owned());
                                    self.get_dependencies(pkgname, false);
                                }
                            }
                        }
                    }
                }
            }
            Err(err) => {
                log::error!("Cannot get package dependencies: {}", err);
                return vec![];
            }
        }

        self.data.clone().into_iter().collect::<Vec<String>>()
    }
}

impl PkgDepTrace for DebPackageTrace {
    fn trace(&mut self, pkgname: String) -> Vec<String> {
        log::info!("Getting dependencies for a package {}", pkgname);

        self.get_dependencies(pkgname, true)
    }
}
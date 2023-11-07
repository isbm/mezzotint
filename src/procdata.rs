use crate::{
    filters::{dirs::PathsDataFilter, intf::DataFilter, resources::ResourcesDataFilter, texts::TextDataFilter},
    profile::Profile,
    rootfs,
    scanner::{binlib::ElfScanner, debpkg::DebPackageScanner, dlst::ContentFormatter, general::Scanner},
};
use std::fs::{self, canonicalize, remove_file, DirEntry, File};
use std::{
    collections::HashSet,
    io::Error,
    os::unix,
    path::{Path, PathBuf},
};

/// Autodependency mode
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Autodeps {
    Undef,
    Free,
    Clean,
    Tight,
}
/// Main processing of profiles or other data
#[derive(Clone)]
pub struct TintProcessor {
    profile: Profile,
    root: PathBuf,
    dry_run: bool,
    autodeps: Autodeps,
    lockfile: PathBuf,
}

impl TintProcessor {
    pub fn new(root: PathBuf) -> Self {
        TintProcessor {
            profile: Profile::default(),
            root,
            dry_run: true,
            autodeps: Autodeps::Free,
            lockfile: PathBuf::from("/.tinted.lock"),
        }
    }

    /// Set configuration from a profile
    pub fn set_profile(&mut self, profile: Profile) -> &mut Self {
        self.profile = profile;
        self
    }

    /// Set dry-run flag (no actual writes on the target image)
    pub fn set_dry_run(&mut self, dr: bool) -> &mut Self {
        self.dry_run = dr;
        self
    }

    /// Set flag for automatic dependency tracing
    pub fn set_autodeps(&mut self, ad: String) -> &mut Self {
        match ad.as_str() {
            "free" => self.autodeps = Autodeps::Free,
            "clean" => self.autodeps = Autodeps::Clean,
            "tight" => self.autodeps = Autodeps::Tight,
            _ => self.autodeps = Autodeps::Undef,
        }

        self
    }

    // Chroot to the mount point
    fn switch_root(&self) -> Result<(), Error> {
        unix::fs::chroot(self.root.to_str().unwrap())?;
        std::env::set_current_dir("/")?;

        Ok(())
    }

    /// Swipe for any broken symlinks.
    fn remove_broken_symlinks(p: &PathBuf) {
        fs::read_dir(p).unwrap().filter_map(|fe| fe.ok()).collect::<Vec<DirEntry>>().into_iter().for_each(|e| {
            if e.path().is_symlink() && canonicalize(e.path()).is_err() {
                log::debug!("Removing broken symlink: {:?}", e.path());
                let _ = remove_file(e.path());
            }

            if e.path().is_dir() {
                TintProcessor::remove_broken_symlinks(&e.path());
            }
        });
    }

    /// After changes are applied, remove all empty directories
    fn remove_empty_dirs(p: &PathBuf) -> Result<bool, Error> {
        let mut empty = true;

        for e in fs::read_dir(p).unwrap() {
            let e = e.unwrap();
            let meta = e.metadata().unwrap();

            if meta.is_dir() {
                let sub_p = e.path();

                if TintProcessor::remove_empty_dirs(&sub_p)? {
                    let _ = fs::remove_dir(&sub_p);
                } else {
                    empty = false;
                }
            }
        }

        TintProcessor::remove_broken_symlinks(p);

        Ok(empty)
    }

    /// Remove files from the image
    fn apply_changes(&self, paths: Vec<PathBuf>) -> Result<(), Error> {
        for p in paths {
            if let Err(err) = fs::remove_file(&p) {
                log::error!("Unable to remove file {}: {}", p.to_str().unwrap(), err);
            }
        }

        TintProcessor::remove_empty_dirs(&PathBuf::from("/"))?;
        File::create(&self.lockfile)?; // Create an empty lock file, indicated mission complete.

        Ok(())
    }

    fn ext_path(p: HashSet<PathBuf>, mut np: HashSet<PathBuf>) -> HashSet<PathBuf> {
        for tgt in p.iter() {
            if tgt.is_symlink() {
                let mut n_tgt = fs::read_link(tgt).unwrap();
                n_tgt = tgt.parent().unwrap().join(&n_tgt);

                if !np.contains(&n_tgt) {
                    np.insert(n_tgt);
                    np.extend(TintProcessor::ext_path(p.clone(), np.clone()));
                }
            }
        }

        np
    }

    // Start tint processor
    pub fn start(&self) -> Result<(), Error> {
        self.switch_root()?;

        if self.lockfile.exists() {
            return Err(Error::new(std::io::ErrorKind::AlreadyExists, "This container seems already tinted."));
        }

        // Paths to keep
        let mut paths: HashSet<PathBuf> = HashSet::default();

        for target_path in self.profile.get_targets() {
            log::debug!("Find binary dependencies for {target_path}");
            paths.extend(ElfScanner::new().scan(Path::new(target_path).to_owned()));

            log::debug!("Find package dependencies for {target_path}");
            // XXX: This will re-scan again and again, if target_path belongs to the same package
            paths.extend(DebPackageScanner::new(self.autodeps).scan(Path::new(target_path).to_owned()));

            // Add the target itself
            paths.insert(Path::new(target_path).to_owned());
        }

        // Scan content of all profile packages (if any)
        // and then let TextDataFilter removes what still should be removed.
        // The idea is to keep parts only relevant to the runtime.
        log::debug!("Filtering packages");
        let pscan = DebPackageScanner::new(Autodeps::Undef);
        for p in self.profile.get_packages() {
            log::debug!("Getting content of package \"{}\"", p);
            paths.extend(pscan.get_package_contents(p.to_string())?);
        }

        log::debug!("Filtering text data");
        TextDataFilter::new(paths.to_owned(), self.profile.to_owned()).filter(&mut paths);

        log::debug!("Filtering directories");
        PathsDataFilter::new(paths.clone().into_iter().collect::<Vec<PathBuf>>(), self.profile.to_owned()).filter(&mut paths);

        // Explicitly keep paths
        // XXX: Support globbing
        paths.extend(self.profile.get_keep_paths());

        // Explicitly knock-out paths
        // XXX: Support globbing
        for p in self.profile.get_prune_paths() {
            paths.remove(&p);
        }

        paths.extend(TintProcessor::ext_path(paths.clone(), HashSet::default()));

        // Remove resources
        log::debug!("Filtering resources");
        ResourcesDataFilter::new(paths.clone().into_iter().collect::<Vec<PathBuf>>(), self.profile.to_owned(), self.autodeps)
            .filter(&mut paths);

        // Scan rootfs
        log::debug!("Scanning existing rootfs");
        let mut p = rootfs::RootFS::new()
            .keep_pds(true)
            .keep_tmp(false)
            .keep_tree(vec![])
            .dissect(paths.clone().into_iter().collect::<Vec<PathBuf>>());
        p.sort();

        let mut paths = paths.into_iter().collect::<Vec<PathBuf>>();
        paths.sort();

        if self.dry_run {
            ContentFormatter::new(&paths).set_removed(&p).format();
        } else {
            self.apply_changes(p)?;
        }

        Ok(())
    }
}

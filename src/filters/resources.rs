use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::profile::Profile;

use super::{defs, intf::DataFilter};

pub struct ResourcesDataFilter {
    data: Vec<PathBuf>,
    remove_archives: bool,
    remove_images: bool, // not blobs (qcow2, raw etc) but images, like JPEG, PNG, XPM...
}

impl ResourcesDataFilter {
    pub fn new(data: Vec<PathBuf>, profile: Profile) -> Self {
        let mut rdf = ResourcesDataFilter { data, remove_archives: false, remove_images: false };
        if profile.filter_arc() {
            log::debug!("Removing archives");
            rdf.remove_archives = true;
        }

        if profile.filter_img() {
            log::debug!("Removing images, pictures, and vector graphics");
            rdf.remove_images = true;
        }

        rdf
    }

    // Is an archive
    fn filter_archives(&self, p: &Path) -> bool {
        if !self.remove_archives {
            return false;
        }

        let p = p.to_str().unwrap();

        for s in defs::ARC_F_EXT {
            if p.ends_with(s) {
                return true;
            }
        }

        false
    }

    /// Is an image (picture)
    fn filter_images(&self, p: &Path) -> bool {
        if !self.remove_images {
            return false;
        }

        let p = p.to_str().unwrap();
        for s in defs::IMG_F_EXT {
            if p.ends_with(s) {
                return true;
            }
        }

        false
    }
}

impl DataFilter for ResourcesDataFilter {
    fn filter(&self, data: &mut HashSet<PathBuf>) {
        let mut out: Vec<PathBuf> = Vec::default();
        for p in &self.data {
            if self.filter_archives(p) || self.filter_images(p) {
                continue;
            }
            out.push(p.to_owned());
        }

        data.clear();
        data.extend(out);
    }
}

use std::{path::{Path, PathBuf}, sync::Arc};

use camino::{Utf8Path, Utf8PathBuf};

pub type ArcPath = Arc<Utf8Path>;

pub trait IntoArcPath {
    fn into_arc_path(self) -> ArcPath;
}

impl IntoArcPath for PathBuf {
    fn into_arc_path(self) -> ArcPath {
        Utf8PathBuf::from_path_buf(self).expect("All paths must be UTF-8").into_arc_path()
    }
}

impl IntoArcPath for &Path {
    fn into_arc_path(self) -> ArcPath {
        Utf8PathBuf::from_path_buf(self.to_owned()).expect("All paths must be UTF-8").into_arc_path()
    }
}

impl IntoArcPath for Utf8PathBuf {
    fn into_arc_path(self) -> ArcPath {
        self.into()
    }
}

impl IntoArcPath for &Utf8Path {
    fn into_arc_path(self) -> ArcPath {
        self.into()
    }
}

impl IntoArcPath for Arc<Path> {
    fn into_arc_path(self) -> ArcPath {
        assert!(Utf8Path::from_path(self.as_ref()).is_some(), "All paths must be UTF-8");
    
        // 2. Consume the original Arc into a raw fat pointer (*const Path)
        let raw_path = Arc::into_raw(self);
    
        // 3. Cast the fat pointer:
        //    `Utf8Path` is #[repr(transparent)] around `str`.
        //    On Unix/Windows, the length metadata of `Path` and `Utf8Path` match
        //    because both measure byte length.
        let raw_utf8 = raw_path as *const Utf8Path;
    
        // 4. Reconstruct the Arc
        // SAFETY:
        // - Checked UTF-8 validity above.
        // - Memory layout and allocation prefix of Arc<Path> and Arc<Utf8Path> are identical.
        // - Ownership was preserved across into_raw -> from_raw without double drops.
        unsafe { Arc::from_raw(raw_utf8) }
    }
}

impl IntoArcPath for Box<Utf8Path> {
    fn into_arc_path(self) -> ArcPath {
        Arc::from(self)
    }
}

impl IntoArcPath for ArcPath {
    fn into_arc_path(self) -> ArcPath {
        self
    }
}

impl IntoArcPath for &ArcPath {
    fn into_arc_path(self) -> ArcPath {
        self.clone()
    }
}





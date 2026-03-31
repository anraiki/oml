use std::path::PathBuf;

/// A resolved filesystem path with addressable sub-properties.
///
/// Backed by POSIX.1-2017 and Win32 UNC path conventions.
/// Any string is a valid path, so this type never fails to construct.
#[derive(Debug, Clone, PartialEq)]
pub struct OmlPath {
    raw: String,
    path: PathBuf,
}

impl OmlPath {
    pub fn new(input: &str) -> Self {
        OmlPath {
            raw: input.to_string(),
            path: PathBuf::from(input),
        }
    }

    /// The original, unmodified path string.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn parent(&self) -> Option<String> {
        self.path
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
    }

    pub fn stem(&self) -> Option<String> {
        self.path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
    }

    pub fn name(&self) -> Option<String> {
        self.path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
    }

    pub fn extension(&self) -> Option<String> {
        self.path
            .extension()
            .map(|s| s.to_string_lossy().into_owned())
    }

    pub fn is_absolute(&self) -> bool {
        self.path.is_absolute()
    }

    pub fn parts(&self) -> Vec<String> {
        self.path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect()
    }

    // --- Mutation ---

    pub fn set_parent(&mut self, parent: &str) {
        let file_name = self.path.file_name().map(|n| n.to_os_string());
        self.path = PathBuf::from(parent);
        if let Some(name) = file_name {
            self.path.push(name);
        }
        self.raw = self.path.to_string_lossy().into_owned();
    }

    pub fn set_stem(&mut self, stem: &str) {
        let parent = self.path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let ext = self.path.extension().map(|e| e.to_os_string());
        let mut new_name = stem.to_string();
        if let Some(ext) = ext {
            new_name.push('.');
            new_name.push_str(&ext.to_string_lossy());
        }
        self.path = parent.join(new_name);
        self.raw = self.path.to_string_lossy().into_owned();
    }

    pub fn set_extension(&mut self, ext: Option<&str>) {
        match ext {
            Some(e) => {
                self.path.set_extension(e);
            }
            None => {
                self.path.set_extension("");
            }
        }
        self.raw = self.path.to_string_lossy().into_owned();
    }
}

impl std::fmt::Display for OmlPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

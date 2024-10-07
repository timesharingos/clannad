use std::{path::Path, vec::IntoIter};

pub enum FileType {
    REGULAR,
    DIRECTORY,
}

pub struct FileInfo {
    // filesystem path
    pub path: String,
    // actual file path
    pub content_path: String,
    // file path symlink points to
    pub symlink_path: Option<String>,
    //file type
    pub file_type: FileType,
}

impl FileInfo {
    pub fn new(
        path: &Path,
        content_path: &Path,
        file_type: FileType,
        symlink_path: Option<&Path>,
    ) -> Self {
        Self {
            path: path.to_str().expect("invalid path").to_owned(),
            content_path: content_path.to_str().expect("invalid path").to_owned(),
            file_type,
            symlink_path: symlink_path.map(|p| p.to_str().expect("invalid path").to_owned()),
        }
    }
}

pub trait Filter: IntoIterator {
    fn new(root: &Path) -> Self;
    fn scan(&mut self);
    fn files(&self) -> &Option<Vec<FileInfo>>;
    fn update(self, root: &Path) -> Self;
}

/// The filter consider all of the files into regular files ignoring symlinks, and
/// only check exisitence of the root path.
/// ```
/// use clannad::filter::{Filter, BasicFilter};
/// use std::path::Path;
///
/// let mut filter = BasicFilter::new(Path::new("resources/normalfolder"));
/// filter.scan();
/// assert_eq!(filter.into_iter().len(), 10);
/// ```
pub struct BasicFilter {
    root: String,
    files: Option<Vec<FileInfo>>,
}

impl BasicFilter {
    fn list_files(&self) -> Option<Vec<FileInfo>> {
        if !Path::new(&self.root).try_exists().is_ok_and(|x| x) {
            return None;
        }
        Some(self.list_files_recursive(Path::new(&self.root)))
    }
    //assume root exists
    fn list_files_recursive(&self, root: &Path) -> Vec<FileInfo> {
        if !root.is_dir() && !root.is_file() {
            return Vec::new();
        }
        if root.is_file() {
            return vec![FileInfo::new(root, root, FileType::REGULAR, None)];
        }
        let mut results = Vec::new();
        results.push(FileInfo::new(root, root, FileType::REGULAR, None));
        for subfile in root.read_dir().expect("invalid directory") {
            match subfile {
                Ok(file) => results.append(&mut self.list_files_recursive(file.path().as_path())),
                Err(_) => {}
            };
        }
        results
    }
}

impl Filter for BasicFilter {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_str().expect("not valid UTF-8 path").to_owned(),
            files: None,
        }
    }

    fn scan(&mut self) {
        self.files = self.list_files();
    }

    fn files(&self) -> &Option<Vec<FileInfo>> {
        &self.files
    }

    fn update(self, root: &Path) -> Self {
        let mut instance = self;
        instance.root = root.to_str().expect("invalid path").to_owned();
        instance.files = None;
        instance
    }
}

impl IntoIterator for BasicFilter {
    type Item = FileInfo;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.unwrap_or(Vec::new()).into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_filter() {
        let mut filter = BasicFilter::new(Path::new("dst"));
        filter.scan();
        assert_eq!(filter.files().is_none(), true);
        filter = filter.update(Path::new("resources/normalfolder"));
        filter.scan();
        assert_eq!(filter.into_iter().len(), 10 as usize);
    }
}

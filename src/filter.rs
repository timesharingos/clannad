use std::{collections::VecDeque, fs, path::Path, vec::IntoIter};

pub enum FileType {
    REGULAR,
    DIRECTORY,
    SYMLINK,
    NONE,
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

pub trait Filter: IntoIterator<Item = FileInfo> {
    fn new(root: &Path) -> Self;
    fn scan(&mut self);
    fn files(&self) -> &Option<Vec<FileInfo>>;
    fn update(self, root: &Path) -> Self;
}

/// The filter consider all of the files into regular files ignoring symlinks, and
/// only check exisitence of the root path.
///
/// Symlink Behavior: transfer symlinks to regular file, does not follow symlinks.
///
/// ```
/// # use clannad::filter::{Filter, BasicFilter};
/// # use std::path::Path;
///
/// let mut filter = BasicFilter::new(Path::new("resources/normalfolder"));
/// filter.scan();
/// assert_eq!(filter.into_iter().len(), 10);
/// ```
pub struct BasicFilter {
    root: String,
    files: Option<Vec<FileInfo>>,
}

/// The filter does not follow the symlink, even if the symlink
/// is broken.
///
/// Symlink Behavior: retain symlinks, but not follow, symlinks pointing to symlinks treated as symlinks as well.
///
/// ```
/// # use clannad::filter::{Filter, SymlinkFilter};
/// # use std::path::Path;
///
/// let mut filter = SymlinkFilter::new(Path::new("resources/normalfolder"));
/// filter.scan();
/// assert_eq!(
///     filter
///         .files()
///         .as_ref()
///         .unwrap()
///         .iter()
///         .filter(|x| x.symlink_path.is_some())
///         .count(),
///     3
/// );
/// assert_eq!(filter.into_iter().len(), 8 as usize);
/// ```
pub struct SymlinkFilter {
    root: String,
    files: Option<Vec<FileInfo>>,
}

/// The filter follows the symlink, and transfer all symlink to copy of what it points to.
///
/// Symlink Behavior: transfer symlink to destination files/directories, and eliminate symlink.
///
/// ```
/// # use clannad::filter::{Filter, SymlinkFollowFilter};
/// # use std::path::Path;
/// let mut filter = SymlinkFollowFilter::new(Path::new("resources/normalsymlink"));
/// filter.scan();
/// assert_eq!(filter.into_iter().len(), 10 as usize);
/// ```
pub struct SymlinkFollowFilter {
    root: String,
    files: Option<Vec<FileInfo>>,
}

impl BasicFilter {
    fn list_files(&self) -> Option<Vec<FileInfo>> {
        let root = self.root.clone();
        if Path::new(&root).is_symlink() {
            return Some(vec![FileInfo::new(
                Path::new(&root),
                Path::new(&root),
                FileType::REGULAR,
                None,
            )]);
        }
        if !Path::new(&self.root).try_exists().is_ok_and(|x| x) {
            return None;
        }
        let mut results = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(root);
        while !queue.is_empty() {
            let root = queue.pop_front().unwrap();
            let root_path = Path::new(&root);
            results.push(FileInfo::new(
                root_path,
                root_path,
                if root_path.is_dir() {
                    FileType::DIRECTORY
                } else {
                    FileType::REGULAR
                },
                None,
            ));
            if root_path.is_file() {
                continue;
            }
            for subfile in root_path.read_dir().unwrap() {
                queue.push_back(subfile.unwrap().path().to_str().unwrap().to_owned());
            }
        }
        Some(results)
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

impl SymlinkFilter {
    fn list_files(&self) -> Option<Vec<FileInfo>> {
        let root = self.root.clone();
        if Path::new(&self.root).is_symlink() {
            return Some(vec![Self::query_fileinfo(&self.root)]);
        }
        if !Path::new(&self.root).try_exists().is_ok_and(|x| x) {
            return None;
        }
        let mut results = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(root);
        while !queue.is_empty() {
            let next = queue.pop_front().expect("unreachable");
            results.push(Self::query_fileinfo(&next));
            match Self::query_next_batch(&next) {
                Some(next) => next.into_iter().for_each(|p| queue.push_back(p)),
                None => {}
            };
        }
        Some(results)
    }

    //assume path exists
    fn query_fileinfo(path: &str) -> FileInfo {
        let abstract_path = Path::new(path);
        if abstract_path.is_symlink() {
            match fs::read_link(abstract_path) {
                Ok(points_to) => FileInfo::new(
                    Path::new(path),
                    Path::new(path),
                    if !points_to.try_exists().is_ok_and(|x| x) {
                        FileType::NONE
                    } else if points_to.is_symlink() {
                        FileType::SYMLINK
                    } else if points_to.is_dir() {
                        FileType::DIRECTORY
                    } else {
                        FileType::REGULAR
                    },
                    Some(points_to.as_path()),
                ),
                Err(_) => unreachable!(),
            }
        } else {
            FileInfo::new(
                abstract_path,
                abstract_path,
                if abstract_path.is_dir() {
                    FileType::DIRECTORY
                } else {
                    FileType::REGULAR
                },
                None,
            )
        }
    }
    fn query_next_batch(path: &str) -> Option<Vec<String>> {
        let abstract_path = Path::new(path);
        if abstract_path.is_symlink() {
            None
        } else if abstract_path.is_file() {
            None
        } else {
            Some(Vec::from_iter(
                abstract_path
                    .read_dir()
                    .expect("invalid path")
                    .filter(|e| e.is_ok())
                    .map(|e| e.unwrap().path().to_str().expect("invalid path").to_owned()),
            ))
        }
    }
}

impl Filter for SymlinkFilter {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_str().expect("invalid path").to_owned(),
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

impl IntoIterator for SymlinkFilter {
    type Item = FileInfo;
    type IntoIter = IntoIter<FileInfo>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.unwrap_or(Vec::new()).into_iter()
    }
}

impl Filter for SymlinkFollowFilter {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_str().expect("invalid path").to_owned(),
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

impl SymlinkFollowFilter {
    fn list_files(&self) -> Option<Vec<FileInfo>> {
        let root = self.root.clone();
        if !Path::new(&root).try_exists().is_ok_and(|x| x) {
            return None;
        }
        let mut queue = VecDeque::new();
        let mut results = Vec::new();
        queue.push_back(root);

        while !queue.is_empty() {
            let root = queue.pop_front().unwrap();
            results.push(Self::query_fileinfo(&root));
            Self::query_next_batch(&root)
                .iter()
                .for_each(|p| queue.push_back(p.to_owned()));
        }
        Some(results)
    }

    fn query_next_batch(path: &str) -> Vec<String> {
        let info = Self::query_fileinfo(path);
        let abstract_path = Path::new(&info.content_path);
        if abstract_path.is_file() {
            vec![]
        } else {
            abstract_path
                .read_dir()
                .unwrap()
                .map(|x| x.unwrap())
                .map(|x| x.path())
                .map(|p| p.to_str().unwrap().to_owned())
                .collect()
        }
    }
    fn query_fileinfo(path: &str) -> FileInfo {
        let abstract_path = Path::new(path);
        if abstract_path.is_symlink() {
            Self::follow_link(path)
        } else {
            FileInfo::new(
                abstract_path,
                abstract_path,
                if abstract_path.is_dir() {
                    FileType::DIRECTORY
                } else {
                    FileType::REGULAR
                },
                None,
            )
        }
    }
    fn follow_link(symlink: &str) -> FileInfo {
        let symlink_path = Path::new(symlink);
        let mut destination_path = symlink_path
            .read_link()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        while Path::new(&destination_path).is_symlink() {
            destination_path = Path::new(&destination_path)
                .read_link()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
        }
        let destination_path = Path::new(symlink_path)
            .parent()
            .unwrap()
            .join(Path::new(&destination_path));
        FileInfo::new(
            symlink_path,
            destination_path.as_path(),
            if destination_path.is_dir() {
                FileType::DIRECTORY
            } else {
                FileType::REGULAR
            },
            None,
        )
    }
}

impl IntoIterator for SymlinkFollowFilter {
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

    #[test]
    fn basic_filter_symlink_root() {
        let mut filter = BasicFilter::new(Path::new("resources/normalsymlink"));
        filter.scan();
        assert_eq!(filter.into_iter().len(), 1 as usize);
    }

    #[test]
    fn symlink_filter() {
        let mut filter = SymlinkFilter::new(Path::new("dst"));
        filter.scan();
        assert_eq!(filter.files().is_none(), true);
        filter = filter.update(Path::new("resources/normalfolder"));
        filter.scan();
        assert_eq!(
            filter
                .files()
                .as_ref()
                .unwrap()
                .iter()
                .filter(|x| x.symlink_path.is_some())
                .count(),
            3
        );
        assert_eq!(filter.into_iter().len(), 8 as usize);
    }

    #[test]
    fn symlink_filter_symlink_root() {
        let mut filter = SymlinkFilter::new(Path::new("resources/normalsymlink"));
        filter.scan();
        assert_eq!(filter.into_iter().len(), 1 as usize);
    }

    #[test]
    fn symlink_follow_filter() {
        let mut filter = SymlinkFollowFilter::new(Path::new("dst"));
        filter.scan();
        assert_eq!(filter.files().is_none(), true);
        filter = filter.update(Path::new("resources/normalfolder"));
        filter.scan();
        assert!(filter
            .files()
            .as_ref()
            .unwrap()
            .iter()
            .all(|x| x.symlink_path.is_none()));
        assert_eq!(filter.into_iter().len(), 10 as usize);
    }

    #[test]
    fn symlink_follow_filter_symlink_root() {
        let mut filter = SymlinkFollowFilter::new(Path::new("resources/normalsymlink"));
        filter.scan();
        assert_eq!(filter.into_iter().len(), 10 as usize);
    }
}

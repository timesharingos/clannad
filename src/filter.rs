use std::{path::Path, vec::IntoIter};

pub trait Filter: IntoIterator {
    fn new(root: &Path) -> Self;
    fn scan(&mut self);
    fn files(&self) -> &Option<Vec<String>>;
}

pub struct BasicFilter {
    root: String,
    files: Option<Vec<String>>,
}

impl BasicFilter {
    fn list_files(&self) -> Option<Vec<String>> {
        if !Path::new(&self.root).try_exists().is_ok_and(|x| x) {
            return None;
        }
        Some(self.list_files_recursive(Path::new(&self.root)))
    }
    //assume root exists
    fn list_files_recursive(&self, root: &Path) -> Vec<String> {
        if !root.is_dir() && !root.is_file() {
            return Vec::new();
        }
        if root.is_file() {
            return vec![root.to_str().expect("not valid UTF-8 path").to_owned()];
        }
        let mut results = Vec::new();
        results.push(root.to_str().expect("not valid UTF-8 path").to_owned());
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

    fn files(&self) -> &Option<Vec<String>> {
        &self.files
    }
}

impl IntoIterator for BasicFilter {
    type Item = String;
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
        let mut filter = BasicFilter::new(Path::new("resources/normalfolder"));
        filter.scan();
        assert_eq!(filter.into_iter().len(), 10 as usize);
    }
}

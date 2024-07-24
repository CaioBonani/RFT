use std::path::{Path, PathBuf};

pub struct CurrentPath {
    current_path: PathBuf,
}

impl CurrentPath {

    pub fn new(initial_path: String) -> Self {

        CurrentPath {
            current_path: PathBuf::from(initial_path),
        }
    }

    pub fn change_path(&mut self, new_path: String) {
        self.current_path = self.current_path.join(new_path);
    }

    pub fn get_path(&self) -> &Path {
        &self.current_path
    }
}
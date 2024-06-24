use std::path::{Path, PathBuf};
use core_allure::AllureDataProvider;

fn main() {
    
}

struct AllureFileSource {
    root_path: PathBuf,
}

impl AllureDataProvider for AllureFileSource {
    async fn get_file_string<P: AsRef<Path>>(&self, path: P) -> String {
        let mut final_path = self.root_path.clone();
        final_path.push(path);
        std::fs::read_to_string(final_path).unwrap()
    }
}
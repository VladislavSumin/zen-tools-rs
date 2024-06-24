use std::path::{Path, PathBuf};
use tracing::Level;
use core_allure::{AllureDataProvider, parse_allure_report};


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();
    
    let allure_source = AllureFileSource {
        root_path: PathBuf::from("./allure-reports")
    };
    let tests_info = parse_allure_report(&allure_source).await;
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

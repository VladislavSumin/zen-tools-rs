use std::future::Future;
use std::path::{Path, PathBuf};
use tracing::{info, Level};
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

    info!("Tests:{:#?}", tests_info);
}

#[derive(Clone)]
struct AllureFileSource {
    root_path: PathBuf,
}


impl AllureDataProvider for AllureFileSource {
    fn get_file_string<P: AsRef<Path> + Send>(&self, path: P) -> impl Future<Output=String> + Send {
        let root_path = self.root_path.clone();
        async move {
            let mut final_path = root_path.clone();
            final_path.push(path);
            std::fs::read_to_string(final_path).unwrap()
        }
    }
}

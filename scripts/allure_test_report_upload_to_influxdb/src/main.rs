use std::future::Future;
use std::path::{Path, PathBuf};
use tracing::{info, Level};
use core_allure::{AllureDataProvider, parse_allure_report};


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // let allure_source = AllureNetworkSource {
    //     base_url: "https://s3.dzeninfra.ru/zen-mobile-allure/master/13614136/zen/ZenApp/build/artifacts/allure_report/allure-reports".to_owned()
    // };

    let allure_source = AllureFileSource {
        root_path: PathBuf::from("./allure-reports")
    };
    let tests_info = parse_allure_report(&allure_source).await;

    info!("Tests:{:#?}", tests_info);
}

#[derive(Clone)]
struct AllureNetworkSource {
    base_url: String,
}

impl AllureDataProvider for AllureNetworkSource {
    fn get_file_string<P: AsRef<Path> + Send>(&self, path: P) -> impl Future<Output=String> + Send {
        let url = format!("{}/{}", &self.base_url, path.as_ref().to_str().unwrap());
        async {
            reqwest::get(url).await.unwrap().text().await.unwrap()
        }
    }
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

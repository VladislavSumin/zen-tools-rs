use std::path::PathBuf;
pub use crate::allure_data_provider::AllureDataProvider;
use crate::json_models::AllureJson;

mod json_models;
mod allure_data_provider;

pub async fn parse_allure_report<T: AllureDataProvider>(data_provider: &T) {
    let allure_path = PathBuf::from("data/packages.json");
    let allure_report = data_provider.get_file_string(allure_path).await;
    let allure_report: AllureJson = serde_json::from_str(&allure_report).unwrap();
}
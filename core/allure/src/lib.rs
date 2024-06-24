#![allow(async_fn_in_trait)]

use std::path::PathBuf;
use tracing::info;

pub use crate::allure_data_provider::AllureDataProvider;
use crate::json_models::AllureJson;

mod json_models;
mod allure_data_provider;

pub async fn parse_allure_report<T: AllureDataProvider>(data_provider: &T) -> Vec<TestInfo> {
    let allure_path = PathBuf::from("data/packages.json");
    let allure_report = data_provider.get_file_string(allure_path).await;
    let allure_report: AllureJson = serde_json::from_str(&allure_report).unwrap();
    let uids = get_test_uids_recursively(&allure_report);
    futures::future::join_all(
        uids.into_iter().map(|uid| {
            tokio::task::spawn(async move { parse_test_info(&uid).await })
        })
    ).await.into_iter()
        .map(|result| { result.unwrap() })
        .collect()
}

async fn parse_test_info(uid: &String) -> TestInfo {
    TestInfo
}

/// Возвращает все uid тестов в данном отчете.
fn get_test_uids_recursively(allure_json: &AllureJson) -> Vec<String> {
    let mut uids: Vec<_> = allure_json.childrens.iter().flat_map(|children| {
        children.iter()
            .flat_map(|child| { get_test_uids_recursively(child) })
            .collect::<Vec<_>>()
    }).collect();
    // Используем поле flaky что бы отличить тесты от пакетов.
    if allure_json.flaky.is_some() {
        uids.push(allure_json.uid.clone());
    }
    uids
}

pub struct TestInfo;
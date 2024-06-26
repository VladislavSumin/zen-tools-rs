//! # core_allure
//! Набор утилит для работы с Allure отчетами.
//!
//! ## Создание источника данных для чтения Allure отчета.
//! Для работы с данными требуется реализация [AllureDataProvider].
//! В библиотеке уже есть две готовые реализации: [AllureFileSource] и [AllureNetworkSource].
//!
//! ## Парсинг Allure отчета.
//! Для чтения отчета необходимо вызвать функцию [parse_allure_report] которая вернет вам
//! список всех тестов в отчете в виде вектора [TestInfo].
//!
//! ## Пример использования
//! ```
//! use core_allure::{AllureFileSource, parse_allure_report};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let allure_data_source = AllureFileSource::new("./allure-reports");
//!     let test_reports = parse_allure_report(&allure_data_source).await?;
//!     println!("Allure test reports: {test_reports:#?}");
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::time::Duration;
use anyhow::Context;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;

pub use crate::allure_data_provider::*;
pub use crate::json_models::AllureTestStatus;
use crate::json_models::{AllureJson, TestInfoJson};

mod json_models;
mod allure_data_provider;

/// Парсит вектор всех тестов находящихся в Allure отчете переданному через [data_provider].
/// Более подробный пример использования описан в документации к крейту.
pub async fn parse_allure_report<T, R, E>(data_provider: &T) -> anyhow::Result<Vec<TestInfo>>
where
    T: AllureDataProvider<R, E>,
    R: AsRef<[u8]>,
    E: std::error::Error + Sync + Send + 'static,
{
    let allure_path = PathBuf::from("data/packages.json");
    let allure_report = data_provider.get_file_content(allure_path).await?;
    let allure_report: AllureJson = serde_json::from_slice(allure_report.as_ref())?;
    let uids = get_test_uids_recursively(&allure_report);

    futures::future::join_all(
        uids
            .into_iter()
            .map(|uid| {
                let data_provider = data_provider.clone();
                tokio::task::spawn(async move { parse_test_info(&uid, &data_provider).await })
            })
    )
        .await
        .into_iter()
        .map(|result| { result? })
        .collect()
}

/// Парсит [TestInfo] соответсвующий переданному [uid].
async fn parse_test_info<T, R, E>(uid: &String, data_provider: &T) -> anyhow::Result<TestInfo>
where
    T: AllureDataProvider<R, E>,
    R: AsRef<[u8]>,
    E: std::error::Error + Sync + Send + 'static,
{
    let test_path = PathBuf::from(format!("data/test-cases/{uid}.json"));
    let test_report = data_provider.get_file_content(test_path).await?;
    let test_report: TestInfoJson = serde_json::from_slice(test_report.as_ref())
        .with_context(|| { format!("Failed to parse test report, uid={}", uid) })?;
    let mut labels: HashMap<_, _> = test_report.labels.iter()
        .map(|label| { (label.name.clone(), label.value.clone()) })
        .collect();
    let test_info = TestInfo {
        full_name: test_report.full_name,
        start_time: DateTime::from_timestamp_millis(test_report.time.start)
            .with_context(|| { format!("unexpected time {}", test_report.time.start) })?,
        duration: Duration::from_millis(test_report.time.duration),
        description: test_report.description,
        status: test_report.status,
        retries_count: test_report.retries_count,
        author: labels.remove("developer").unwrap_or_else(|| { "<no_author>".to_owned() }),
        team: labels.remove("suite").unwrap_or_else(|| { "<no_team>".to_owned() }),
        host: labels.remove("host").unwrap_or_else(|| { "<no_host>".to_owned() }),
        retries: test_report.extra.retries.iter().map(|retry_info| {
            let retry_info = RetryInfo {
                start_time: DateTime::from_timestamp_millis(retry_info.time.start)
                    .with_context(|| { format!("unexpected time {}", test_report.time.start) })?,
                duration: Duration::from_millis(retry_info.time.duration),
                status: retry_info.status,
            };
            Ok(retry_info)
        }).collect::<anyhow::Result<Vec<_>>>()?,
    };

    Ok(test_info)
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

#[derive(Debug)]
pub struct TestInfo {
    /// Полное имя теста, пакет + имя класса + имя метода теста.
    pub full_name: String,
    /// Время старта теста.
    pub start_time: DateTime<Utc>,
    /// Продолжительность выполнения теста.
    pub duration: Duration,
    /// Описание теста.
    pub description: Option<String>,
    /// Статус выполнения тетса.
    pub status: AllureTestStatus,
    /// Количество повторных попыток запуска теста. (при успехе с первого раза будет равно 0).
    pub retries_count: u32,
    /// Ник автора теста.
    pub author: String,
    /// Команда которой принадлежит тест.
    pub team: String,
    /// Хост на котором был запущен тест.
    pub host: String,

    pub retries: Vec<RetryInfo>,
}

#[derive(Debug)]
pub struct RetryInfo {
    pub start_time: DateTime<Utc>,
    pub duration: Duration,
    pub status: AllureTestStatus,
}

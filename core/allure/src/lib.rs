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
//! async fn main() {
//!     let allure_data_source = AllureFileSource::new("./allure-reports");
//!     let test_reports = parse_allure_report(&allure_data_source).await;
//!     println!("Allure test reports: {test_reports:#?}");
//! }
//! ```

use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::time::Duration;
use chrono::{DateTime, Utc};
use thiserror::Error;

pub use crate::allure_data_provider::*;
use crate::Error::TimeFormat;
use crate::json_models::{AllureJson, AllureTestStatus, TestInfoJson};

mod json_models;
mod allure_data_provider;

pub type Result<T, E> = std::result::Result<T, Error<E>>;

pub async fn parse_allure_report<T: AllureDataProvider<E>, E: Send + 'static>(data_provider: &T) -> Result<Vec<TestInfo>, E> {
    let allure_path = PathBuf::from("data/packages.json");
    let allure_report = data_provider.get_file_string(allure_path).await
        .map_err(|e| { Error::DataSource(e) })?;
    let allure_report: AllureJson = serde_json::from_str(&allure_report)?;
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
        .map(|result| {
            result
                .map_err(|e| { Error::from(e) })
                .and_then(|e| { e })
        })
        .collect()
}

async fn parse_test_info<T: AllureDataProvider<E>, E: Send + 'static>(uid: &String, data_provider: &T) -> Result<TestInfo, E> {
    let test_path = PathBuf::from(format!("data/test-cases/{uid}.json"));
    let test_report = data_provider.get_file_string(test_path).await
        .map_err(|e| { Error::DataSource(e) })?;
    let test_report: TestInfoJson = serde_json::from_str(&test_report)?;
    let mut labels: HashMap<_, _> = test_report.labels.iter()
        .map(|label| { (label.name.clone(), label.value.clone()) })
        .collect();
    let test_info = TestInfo {
        full_name: test_report.full_name,
        start_time: DateTime::from_timestamp_millis(test_report.time.start).ok_or(TimeFormat)?,
        duration: Duration::from_millis(test_report.time.duration),
        description: test_report.description,
        status: test_report.status,
        retries_count: test_report.retries_count,
        author: labels.remove("developer").unwrap_or_else(|| { "<no_author>".to_owned() }),
        team: labels.remove("suite").unwrap_or_else(|| { "<no_team>".to_owned() }),
        host: labels.remove("host").unwrap_or_else(|| { "<no_host>".to_owned() }),
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
}

/// Ошибки, которые могут случиться при парсинге Allure отчета.
/// [E] тип ошибки возвращаемый [AllureDataProvider]
#[derive(Error, Debug)]
pub enum Error<E> {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
    #[error("Invalid timestamp format")]
    TimeFormat,
    #[error(transparent)]
    DataSource(E),
}

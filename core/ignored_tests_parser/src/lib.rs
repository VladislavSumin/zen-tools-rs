//! # core_ignored_test_parser
//! Парсер для поиска заигноренных тестов.
//! 
//! ## Пример использования:
//! ```
//! use core_ignored_tests_parser::parse_ignored_tests;
//!
//! #[tokio::main]
//! async fn main() {
//!     let ignored_tests = parse_ignored_tests("./path/to/test/root").await;
//!     println!("Ignored tests {ignored_tests:#?}");
//! }
//! ```
use std::fs;
use std::path::Path;
use std::process::Command;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

lazy_static! {
    static ref IGNORE_ANNOTATION_REGEX: Regex = Regex::new("@Ignore(\\(\"(.+)\"\\))?$").unwrap();
    static ref DEVELOPER_ANNOTATION_REGEX: Regex = Regex::new("@Developer\\(Developers\\.(.+)\\)").unwrap();
    static ref TEST_MODULE_ANNOTATION_REGEX: Regex = Regex::new("@TestModule\\(TestModules\\.(.+)\\)").unwrap();
    static ref GIT_TIME_REGEX: Regex = Regex::new("author-time ([0-9]+)").unwrap();
}

pub async fn parse_ignored_tests<P: AsRef<Path>>(path: P) -> Vec<IgnoreInfo> {
    futures::future::join_all(
        WalkDir::new(path)
            .into_iter()
            .map(|x| { x.unwrap() })
            .filter(|file| { file.file_type().is_file() })
            .filter(|file| { file.file_name().to_string_lossy().ends_with(".kt") })
            .map(|file| {
                tokio::task::spawn(async move { process_file(file.path()) })
            })
    ).await
        .into_iter()
        .filter_map(|result| { result.unwrap() })
        .collect()
}

/// Анализирует переданный файл, ищет там ignored тест и возвращает информацию если нашла.
fn process_file(path: &Path) -> Option<IgnoreInfo> {
    let file_content = fs::read_to_string(path).unwrap();

    // Ищем Ignore аннотацию в файле.
    let ignore_match = file_content.lines()
        .enumerate()
        .filter_map(|(index, str)| { IGNORE_ANNOTATION_REGEX.captures(str).map(|captures| { (index, captures) }) })
        .next();

    // Если нашли игнор возвращаем информацию о тесте.
    if ignore_match.is_some() {
        let author = file_content.lines()
            .filter_map(|x| { DEVELOPER_ANNOTATION_REGEX.captures(x) })
            .next().map(|captures| { captures.get(1).unwrap().as_str().to_string() });

        let test_module = file_content.lines()
            .filter_map(|x| { TEST_MODULE_ANNOTATION_REGEX.captures(x) })
            .next().map(|captures| { captures.get(1).unwrap().as_str().to_string() });

        let (ignore_line_index, ignore_captures) = ignore_match.unwrap();

        // Ищем дату когда была поставлена аннотация.
        let ignore_date = get_line_modification_time(path, ignore_line_index + 1);

        Some(IgnoreInfo {
            file_name: path.file_name().unwrap().to_string_lossy().to_string(),
            comment: ignore_captures.get(2).map(|t| { t.as_str().to_string() }),
            author,
            test_module,
            ignore_date,
        })
    } else {
        None
    }
}

/// Возвращает дату модификации переданной строки в переданном файле используя для этого git blame.
/// Обратите внимание это не индекс строки, а ее номер. Строки нумеруются начиная с 1, а не с нуля.
fn get_line_modification_time(file: &Path, line_number: usize) -> DateTime<Utc> {
    let file = fs::canonicalize(file).unwrap();
    let result = Command::new("git")
        .current_dir(file.parent().unwrap())
        .arg("blame")
        .arg("--date=raw")
        .arg("--porcelain")
        .arg("-L").arg(format!("{},{}", line_number, line_number))
        .arg(file)
        .output()
        .unwrap();

    if !result.status.success() {
        let a = std::str::from_utf8(&result.stderr).unwrap();
        println!("git error = {a}");
    }
    assert!(result.status.success(), "git failed");

    let output = std::str::from_utf8(&result.stdout).unwrap();
    let time = output.lines()
        .filter_map(|line| { GIT_TIME_REGEX.captures(line) })
        .next().unwrap().get(1).unwrap().as_str().to_string().parse().unwrap();
    DateTime::from_timestamp(time, 0).unwrap()
}

/// Содержит информацию о тесте в Ignore.
#[derive(Debug, Serialize)]
pub struct IgnoreInfo {
    /// Имя файла.
    pub file_name: String,
    /// Опциональный комментарий (причина указанная в аннотации @Ignore).
    pub comment: Option<String>,
    /// Автор теста
    pub author: Option<String>,
    /// Тестовый модуль
    pub test_module: Option<String>,
    /// Дата установки аннотации игнор
    pub ignore_date: DateTime<Utc>,
}
use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use tracing::{info, Level};
use core_allure::{AllureFileSource, parse_allure_report, TestInfo};


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let allure_source = AllureFileSource::new("./allure-reports");
    let tests_info = parse_allure_report(&allure_source).await;

    info!("Tests:{:#?}", tests_info);
    let aggregated_report = make_aggregated_test_report(&tests_info, "master");
    info!("Aggregated report: {aggregated_report:#?}");
}

/// Собирает агрегированный отчет по тестам.
///
/// [branch] ветка на которой запускался этот тестовый прогон.
fn make_aggregated_test_report(tests: &Vec<TestInfo>, branch: &str) -> IDAggregatedTestReport {
    // Для простоты берем время старта первого теста, нам хватит такой точности.
    let time = tests.first().unwrap().start_time;

    let mut report = IDAggregatedTestReport {
        time,
        is_success: 1,
        branch: branch.to_owned(),
        ..Default::default()
    };

    tests.iter().for_each(|test| {
        report.total_tests += 1;

        if !test.status.is_success() {
            report.failed_tests += 1;
            report.failed_tries += 1;
            report.is_success = 0;
        }

        report.total_tries += test.retries_count + 1;
        report.failed_tries += test.retries_count;
    });

    report
}

#[derive(InfluxDbWriteable, Default, Debug)]
struct IDAggregatedTestReport {
    /// Время прогона
    time: DateTime<Utc>,
    /// Общее количество тестов
    total_tests: u32,
    /// Проваленных тестов
    failed_tests: u32,

    /// Всего попыток
    total_tries: u32,
    /// Проваленых попыток
    failed_tries: u32,

    /// Общее состояние прогона. Поле u32 так как с такими данными проще работать на стороне
    /// influxdb. Bool там не агрегируется сами по себе приходится явно обрабатывать этот сценарий.
    /// Хоть это поле и вычисляемое, но его удобно вычислить заранее что бы упростить итоговые
    /// запросы к базе.
    is_success: u32,

    /// Ветка на которой запускались тесты
    #[influxdb(tag)]
    branch: String,

    // Успешых тестов, нет смысла писать, поле вычисляемое
    // success_tests: u32,
    // Успешных попыток, нет смысла писать, так как их количество всегда равно success_tests
    // success_tries: u32,
}

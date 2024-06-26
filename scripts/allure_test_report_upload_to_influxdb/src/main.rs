use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use tokio::time::Instant;
use tracing::{info, Level};
use core_allure::{AllureFileSource, AllureTestStatus, parse_allure_report, TestInfo};


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let start_time = Instant::now();

    let allure_source = AllureFileSource::new("./allure-reports");
    let tests_info = parse_allure_report(&allure_source).await.unwrap();
    // let time = tests_info.first().unwrap().start_time;

    // let test_report: Vec<_> = tests_info.iter().map(|report| {
    //     IDTestReport::from(report, time, "master")
    // }).collect();
    // info!("Per test report: {test_report:#?}");

    let aggregated_report = make_aggregated_test_report(&tests_info, "master");
    info!("Aggregated report: {aggregated_report:#?}");

    info!("Process time {:?}", start_time.elapsed());
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

    tests.iter().for_each(|test_info| {
        match test_info.status {
            AllureTestStatus::Passed => {
                report.passed_tests += 1;
                report.passed_tries += 1;
            }
            AllureTestStatus::Failed => {
                report.failed_tests += 1;
                report.failed_tries += 1;
            }
            AllureTestStatus::Broken => {
                report.broken_tests += 1;
                report.broken_tries += 1;
            }
            AllureTestStatus::Unknown => {
                report.unknown_tests += 1;
                report.unknown_tries += 1;
            }
        }

        if !test_info.status.is_success() {
            report.is_success = 0;
        }

        test_info.retries.iter().for_each(|retry_info| {
            match retry_info.status {
                AllureTestStatus::Passed => {
                    report.passed_tries += 1;
                }
                AllureTestStatus::Failed => {
                    report.failed_tries += 1;
                }
                AllureTestStatus::Broken => {
                    report.broken_tries += 1;
                }
                AllureTestStatus::Unknown => {
                    report.unknown_tries += 1;
                }
            }
        })
    });

    report
}

#[derive(InfluxDbWriteable, Default, Debug)]
struct IDAggregatedTestReport {
    /// Время прогона.
    time: DateTime<Utc>,

    passed_tests: u32,
    failed_tests: u32,
    broken_tests: u32,
    unknown_tests: u32,

    passed_tries: u32,
    failed_tries: u32,
    broken_tries: u32,
    unknown_tries: u32,

    /// Общее состояние прогона. Поле u32 так как с такими данными проще работать на стороне
    /// influxdb. Bool там не агрегируется сами по себе приходится явно обрабатывать этот сценарий.
    /// Хоть это поле и вычисляемое, но его удобно вычислить заранее что бы упростить итоговые
    /// запросы к базе.
    is_success: u32,

    /// Ветка на которой запускались тесты.
    #[influxdb(tag)]
    branch: String,
}

/// Отдельный отчет по каждому тесту в прогоне.
#[derive(InfluxDbWriteable, Debug)]
struct IDTestReport {
    /// Время прогона. Обратите внимание, для удобства работы с данными сюда пишется время
    /// прогона всех тестов в отчете, а не каждого теста в отдельности.
    time: DateTime<Utc>,

    /// Общее состояние теста после всех попыток.
    is_success: u32,

    /// Общее количество попыток запуска теста (минимум одна).
    total_tries: u32,

    /// Время прогона последней попытки
    duration: u64,

    /// Полное имя теста (пакет + имя класса + имя метода).
    #[influxdb(tag)]
    name: String,

    /// Ветка на которой запускались тесты.
    #[influxdb(tag)]
    branch: String,

    /// Ник автора теста. (не ник в телеге).
    #[influxdb(tag)]
    author: String,

    /// Команда которой принадлежит тест.
    #[influxdb(tag)]
    team: String,

    /// Хост на котором выполнялся данный тест. Возможно не очень полезно, но мало ли.
    #[influxdb(tag)]
    host: String,
}

impl IDTestReport {
    fn from<B: Into<String>>(test_report: &TestInfo, time: DateTime<Utc>, branch: B) -> Self {
        Self {
            time,
            is_success: test_report.status.is_success().into(),
            total_tries: test_report.retries_count + 1,
            duration: test_report.duration.as_millis() as u64,
            name: test_report.full_name.clone(),
            branch: branch.into(),
            author: test_report.author.clone(),
            team: test_report.team.clone(),
            host: test_report.host.clone(),
        }
    }
}
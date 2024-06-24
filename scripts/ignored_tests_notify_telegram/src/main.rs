use std::fmt::Debug;
use std::path::PathBuf;
use std::time::Instant;
use chrono::Utc;
use tracing::{info, Level};
use clap::Parser;

use core_ignored_tests_parser::parse_ignored_tests;

#[tokio::main]
async fn main() {
    let start = Instant::now();

    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();
    info!("Starting...");

    let test_path = args.test_path.clone();
    let current_time = Utc::now();

    let old_tests: Vec<_> = parse_ignored_tests(test_path).await.into_iter()
        .filter(|ignore_info| {
            (current_time - ignore_info.ignore_date).num_days() > 270
        }).collect();

    let msg = if old_tests.len() > 0 {
        let mut msg = "".to_owned();

        msg.push_str(&format!("Найдены тесты заигноренные больше 270 дней назад, в количестве {} штук!\n", old_tests.len()));
        msg.push_str("Просьба починить тесты или удалить их если они не нужны.\n");
        msg.push_str("Список тестов:\n");
        old_tests.into_iter().for_each(|ignore_info| {
            msg.push_str(
                &format!("{}, @{}\n",
                         ignore_info.file_name,
                         ignore_info.author.unwrap_or_else(|| {"<no_author>".to_owned()})
                )
            )
        });

        msg
    } else {
        "Нет тестов заигноренных более 270 дней назад.".to_owned()
    };


    info!("Msg: {}", msg);
    info!("Calculation time {:?}", start.elapsed());
    info!("Done!");
}

/// This script collects information about ignored tests and sent info to telegram chat.
#[derive(Parser, Debug)]
struct Args {
    /// Path to test root.
    test_path: PathBuf,
}

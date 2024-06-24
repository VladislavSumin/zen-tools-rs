use std::fmt::Debug;
use std::path::PathBuf;
use std::time::Instant;
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

    let mut writter = csv::Writer::from_path("ignored_tests.csv").unwrap();

    parse_ignored_tests(test_path).await.iter().for_each(|ignore_info| {
        writter.serialize(ignore_info).unwrap();
    });

    info!("Calculation time {:?}", start.elapsed());
    info!("Done!");
}

/// This script collects information about ignored tests and mage csv table with result.
#[derive(Parser, Debug)]
struct Args {
    /// Path to test root.
    test_path: PathBuf,
}

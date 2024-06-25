use serde::Deserialize;

/// Отчет Allure в json формате.
#[derive(Deserialize, Debug)]
pub struct AllureJson {
    pub uid: String,
    #[serde(rename = "children")]
    pub childrens: Option<Vec<AllureJson>>,
    pub flaky: Option<bool>,
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestInfoJson {
    pub full_name: String,
    pub time: AllureTimeJson,
    pub description: Option<String>,
    pub status: AllureTestStatus,
    pub retries_count: u32,
}


#[derive(Deserialize, Debug)]
pub struct AllureTimeJson {
    pub start: i64,
    pub duration: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AllureTestStatus {
    Passed,
    Failed,
    Unknown,
}

impl AllureTestStatus {
    pub fn is_success(&self) -> bool {
        match self {
            AllureTestStatus::Passed => { true }
            AllureTestStatus::Failed => { false }
            AllureTestStatus::Unknown => { false }
        }
    }
}
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
    pub labels: Vec<AllureLabelJson>,
}


#[derive(Deserialize, Debug)]
pub struct AllureTimeJson {
    pub start: i64,
    pub duration: u64,
}

#[derive(Deserialize, Debug)]
pub struct AllureLabelJson {
    pub name: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct AllureJsonExtra {
    pub retries: Vec<AllureJsonExtraRetry>,
}

#[derive(Deserialize, Debug)]
pub struct AllureJsonExtraRetry {
    pub uid: String,
    pub status: AllureTestStatus,
    pub tine: AllureTimeJson,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AllureTestStatus {
    /// Green
    Passed,
    /// Red
    Failed,
    /// Yellow
    Broken,
    /// Violet
    Unknown,
}

impl AllureTestStatus {
    pub fn is_success(&self) -> bool {
        match self {
            AllureTestStatus::Passed => { true }
            AllureTestStatus::Failed => { false }
            AllureTestStatus::Broken => { false }
            AllureTestStatus::Unknown => { false }
        }
    }
}
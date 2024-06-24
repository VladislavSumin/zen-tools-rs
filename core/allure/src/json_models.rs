use serde::Deserialize;

/// Отчет Allure в json формате.
#[derive(Deserialize, Debug)]
pub struct AllureJson {
    pub uid: String,
    #[serde(rename = "children")]
    pub childrens: Option<Vec<AllureJson>>,
    pub flaky: Option<bool>,
}
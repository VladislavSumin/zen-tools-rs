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
}

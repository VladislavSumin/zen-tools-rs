use serde::Deserialize;

/// Отчет Allure в json формате.
#[derive(Deserialize, Debug)]
pub(crate) struct AllureJson {
    uid: String,
    #[serde(rename = "children")]
    childrens: Option<Vec<AllureJson>>,
}
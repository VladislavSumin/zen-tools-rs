use std::future::Future;
use std::path::{Path, PathBuf};
use bytes::Bytes;

/// Источник данных для чтения allure отчета.
///
/// [R] тип данных возвращаемый при загрузке данных.
/// [E] тип ошибки которая может произойти при загрузке данных.
pub trait AllureDataProvider<R, E>: Clone + Send + Sync + 'static
where
    R: AsRef<[u8]>,
    E: Send + 'static,
{
    /// Предоставляет контент нужного файла.
    /// [path] должен быть всегда относительным, относительно root папки отчета.
    fn get_file_content<P: AsRef<Path> + Send>(&self, path: P) -> impl Future<Output=Result<R, E>> + Send;
}


/// Сетевой источник данных.
#[derive(Clone)]
pub struct AllureNetworkSource {
    base_url: String,
}

impl AllureDataProvider<Bytes, reqwest::Error> for AllureNetworkSource {
    fn get_file_content<P: AsRef<Path> + Send>(&self, path: P) -> impl Future<Output=Result<Bytes, reqwest::Error>> + Send {
        // Тут все же ожидаем что path это валидная UTF-8 строка и паникуем если это не так.
        let url = format!("{}/{}", &self.base_url, path.as_ref().to_str().unwrap());
        async {
            reqwest::get(url).await?.bytes().await
        }
    }
}

impl AllureNetworkSource {
    pub fn new<T: Into<String>>(base_url: T) -> Self {
        Self {
            base_url: base_url.into()
        }
    }
}

/// Файловый источник данных.
#[derive(Clone)]
pub struct AllureFileSource {
    root_path: PathBuf,
}

impl AllureDataProvider<Vec<u8>, std::io::Error> for AllureFileSource {
    fn get_file_content<P: AsRef<Path> + Send>(&self, path: P) -> impl Future<Output=Result<Vec<u8>, std::io::Error>> + Send {
        let root_path = self.root_path.clone();
        async move {
            let mut final_path = root_path.clone();
            final_path.push(path);
            std::fs::read(final_path)
        }
    }
}

impl AllureFileSource {
    pub fn new<T: Into<PathBuf>>(root_path: T) -> Self {
        Self {
            root_path: root_path.into()
        }
    }
}
use std::future::Future;
use std::path::{Path, PathBuf};

pub trait AllureDataProvider<E: Send + 'static>: Clone + Send + Sync + 'static {
    /// Предоставляет контент нужного файла.
    /// [path] должен быть всегда относительным, относительно root папки отчета.
    fn get_file_string<P: AsRef<Path> + Send>(&self, path: P) -> impl Future<Output=Result<String, E>> + Send;
}


/// Сетевой источник данных.
#[derive(Clone)]
pub struct AllureNetworkSource {
    base_url: String,
}

impl AllureDataProvider<reqwest::Error> for AllureNetworkSource {
    fn get_file_string<P: AsRef<Path> + Send>(&self, path: P) -> impl Future<Output=Result<String, reqwest::Error>> + Send {
        let url = format!("{}/{}", &self.base_url, path.as_ref().to_str().unwrap());
        async {
            reqwest::get(url).await?.text().await
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

impl AllureDataProvider<std::io::Error> for AllureFileSource {
    fn get_file_string<P: AsRef<Path> + Send>(&self, path: P) -> impl Future<Output=Result<String, std::io::Error>> + Send {
        let root_path = self.root_path.clone();
        async move {
            let mut final_path = root_path.clone();
            final_path.push(path);
            std::fs::read_to_string(final_path)
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
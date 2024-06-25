use std::future::Future;
use std::path::{Path, PathBuf};

pub trait AllureDataProvider: Clone + Send + Sync + 'static {
    /// Предоставляет контент нужного файла.
    /// path должен быть всегда относительным, относительно root папки отчета.
    fn get_file_string<P: AsRef<Path> + Send>(&self, path: P) -> impl Future<Output=String> + Send;
}


/// Сетевой источник данных.
#[derive(Clone)]
pub struct AllureNetworkSource {
    base_url: String,
}

impl AllureDataProvider for AllureNetworkSource {
    fn get_file_string<P: AsRef<Path> + Send>(&self, path: P) -> impl Future<Output=String> + Send {
        let url = format!("{}/{}", &self.base_url, path.as_ref().to_str().unwrap());
        async {
            reqwest::get(url).await.unwrap().text().await.unwrap()
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

impl AllureDataProvider for AllureFileSource {
    fn get_file_string<P: AsRef<Path> + Send>(&self, path: P) -> impl Future<Output=String> + Send {
        let root_path = self.root_path.clone();
        async move {
            let mut final_path = root_path.clone();
            final_path.push(path);
            std::fs::read_to_string(final_path).unwrap()
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
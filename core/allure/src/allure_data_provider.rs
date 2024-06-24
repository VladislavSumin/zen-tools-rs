use std::future::Future;
use std::path::Path;

pub trait AllureDataProvider: Clone + Send + 'static {
    /// Предоставляет контент нужного файла.
    /// path должен быть всегда относительным, относительно root папки отчета.
    fn get_file_string<P: AsRef<Path>+Send>(&self, path: P) -> impl Future<Output=String> + Send;
}

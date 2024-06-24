use std::path::Path;

pub trait AllureDataProvider {
    /// Предоставляет контент нужного файла.
    /// path должен быть всегда относительным, относительно root папки отчета.
    async fn get_file_string<P: AsRef<Path>>(&self, path: P) -> String;
}

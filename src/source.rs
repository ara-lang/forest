use std::path::PathBuf;
use walkdir::WalkDir;

use crate::config::Config;
use crate::error::Error;
use crate::ARA_SOURCE_EXTENSION;

#[derive(Debug)]
pub struct SourceFilesCollector<'a> {
    config: &'a Config,
}

impl<'a> SourceFilesCollector<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn collect(&self) -> Result<Vec<PathBuf>, Error> {
        let mut paths = vec![&self.config.source];
        paths.extend(&self.config.definitions);

        let mut files = Vec::new();
        for path in paths {
            let path = &self.config.root.join(path);
            if !path.is_dir() {
                return Err(Error::InvalidPath(format!(
                    "{} must be a directory and be relative to the project root directory.",
                    path.display(),
                )));
            }
            for entry in WalkDir::new(path) {
                let entry = entry?;
                if entry.file_type().is_file()
                    && entry.path().extension() == Some(ARA_SOURCE_EXTENSION.as_ref())
                {
                    files.push(entry.into_path());
                }
            }
        }

        Ok(files)
    }
}

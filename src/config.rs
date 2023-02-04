use std::path::PathBuf;

use crate::logger::Logger;

#[derive(Debug)]
pub struct Config {
    pub root: PathBuf,
    pub source: PathBuf,
    pub definitions: Vec<PathBuf>,
    pub cache: Option<PathBuf>,
    pub threads: usize,
    pub logger: Option<Logger>,
}

impl Config {
    pub fn new<R: Into<String>>(root: R) -> Self {
        Self {
            root: PathBuf::from(root.into()),
            source: PathBuf::from(String::default()),
            definitions: Vec::new(),
            cache: None,
            threads: num_cpus::get(),
            logger: None,
        }
    }

    #[must_use]
    pub fn with_source<S: Into<String>>(mut self, source: S) -> Self {
        self.source = PathBuf::from(source.into());

        self
    }

    #[must_use]
    pub fn with_definitions<D: Into<String>>(mut self, definitions: Vec<D>) -> Self {
        self.definitions = definitions
            .into_iter()
            .map(|definition| PathBuf::from(definition.into()))
            .collect();

        self
    }

    #[must_use]
    pub fn with_cache_directory<C: Into<String>>(mut self, cache_dir: C) -> Self {
        let path = PathBuf::from(cache_dir.into());

        if path.is_relative() {
            self.cache = Some(self.root.join(path));
        } else {
            self.cache = Some(path);
        }

        self
    }

    #[must_use]
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads;

        self
    }

    #[must_use]
    pub fn with_logger(mut self, logger: Logger) -> Self {
        self.logger = Some(logger);

        self
    }
}

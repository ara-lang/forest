use std::path::PathBuf;

use crate::hash::ContentHasher;
use crate::hash::FxHasher;
use crate::logger::Logger;
use crate::serializer::BincodeSerializer;
use crate::serializer::Serializer;

pub struct Config {
    pub root: PathBuf,
    pub source: PathBuf,
    pub definitions: Vec<PathBuf>,
    pub cache: Option<PathBuf>,
    pub threads: usize,
    pub logger: Option<Logger>,
    pub hasher: Box<dyn ContentHasher>,
    pub serializer: Box<dyn Serializer>,
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
            hasher: Box::new(FxHasher::new()),
            serializer: Box::new(BincodeSerializer::new()),
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

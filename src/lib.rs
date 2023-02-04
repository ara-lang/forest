use std::fs;
use std::path::PathBuf;
use std::thread;

use ara_parser::tree::Tree;
use ara_parser::tree::TreeMap;
use ara_reporting::Report;
use ara_source::source::Source;
use ara_source::SourceMap;

use crate::config::Config;
use crate::error::Error;
use crate::hash::FxHasher;
use crate::serializer::BincodeSerializer;
use crate::source::SourceFilesCollector;
use crate::tree::TreeBuilder;

pub mod config;
pub mod error;
pub(crate) mod hash;
pub mod logger;
pub(crate) mod serializer;
pub mod source;
pub(crate) mod tree;

pub(crate) const ARA_SOURCE_EXTENSION: &str = "ara";
pub(crate) const ARA_DEFINITION_EXTENSION: &str = "d.ara";
pub(crate) const ARA_CACHED_SOURCE_EXTENSION: &str = "ara.cache";

#[derive(Debug)]
pub struct Forest {
    pub source: SourceMap,
    pub tree: TreeMap,
}

impl Forest {
    pub fn new(source: SourceMap, tree: TreeMap) -> Self {
        Self { source, tree }
    }
}

pub struct Parser<'a> {
    pub config: &'a Config,
    tree_builder: TreeBuilder<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(config: &'a Config) -> Self {
        let tree_builder = TreeBuilder::new(
            config,
            Box::new(FxHasher::new()),
            Box::new(BincodeSerializer::new()),
        );

        Self {
            config,
            tree_builder,
        }
    }

    pub fn parse(&self) -> Result<Forest, Box<Report>> {
        self.init_logger().map_err(|error| Box::new(error.into()))?;

        let (sources, trees) =
            thread::scope(|scope| -> Result<(Vec<Source>, Vec<Tree>), Box<Report>> {
                self.create_cache_dir()
                    .map_err(|error| Box::new(error.into()))?;

                let files = SourceFilesCollector::new(self.config)
                    .collect()
                    .map_err(|error| Box::new(error.into()))?;

                if files.is_empty() {
                    return Ok((Vec::new(), Vec::new()));
                }

                let threads_count = self.threads_count(files.len());
                let chunks = files
                    .chunks(files.len() / threads_count)
                    .map(Vec::from)
                    .collect::<Vec<Vec<PathBuf>>>();

                let mut threads = Vec::with_capacity(threads_count);
                for chunk in chunks.into_iter() {
                    threads.push(scope.spawn(
                        move || -> Result<Vec<(Source, Tree)>, Box<Report>> {
                            let mut source_tree = Vec::with_capacity(chunk.len());
                            for source_path in chunk {
                                let (source, tree) = self
                                    .tree_builder
                                    .build(&source_path)
                                    .map_err(|error| match error {
                                        Error::ParseError(report) => report,
                                        _ => Box::new(error.into()),
                                    })?;
                                source_tree.push((source, tree));
                            }

                            Ok(source_tree)
                        },
                    ));
                }

                let mut result = Vec::new();
                for handle in threads {
                    result.extend(handle.join().unwrap()?);
                }
                let (sources, trees) = result.into_iter().unzip();

                Ok((sources, trees))
            })?;

        Ok(Forest::new(SourceMap::new(sources), TreeMap::new(trees)))
    }

    fn threads_count(&self, files_len: usize) -> usize {
        if self.config.threads > files_len {
            files_len
        } else {
            self.config.threads
        }
    }

    fn create_cache_dir(&self) -> Result<(), Error> {
        if self.config.cache.is_some() {
            fs::create_dir_all(self.config.cache.as_ref().unwrap())?;
        }

        Ok(())
    }

    fn init_logger(&self) -> Result<(), Error> {
        if self.config.logger.is_some() {
            self.config.logger.as_ref().unwrap().init()?
        }

        Ok(())
    }
}

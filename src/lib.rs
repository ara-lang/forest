use ara_parser::parser;
use ara_parser::tree::TreeMap;
use ara_source::loader;
use ara_source::SourceMap;

use crate::config::Config;

pub mod config;

pub struct Forest {
    pub source: SourceMap,
    pub tree: TreeMap,
}

pub struct Parser {
    pub config: Config,
}

impl Parser {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn parse(&self) -> Result<Forest, String> {
        let mut threads = Vec::with_capacity(self.config.threads);

        let source_map = loader::load_directories(&self.config.root, {
            let mut directories = self.config.definitions.clone();
            directories.push(self.config.source.clone());

            directories
        })
        .expect("Failed to load source map");

        // split the sources into N chunks, where N is the number of threads
        let chunk_size = source_map.sources.len() / self.config.threads;
        let chunks: Vec<Vec<ara_source::source::Source>> = source_map
            .sources
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        for chunk in chunks {
            threads.push(std::thread::spawn(move || {
                let map = SourceMap::new(chunk);
                parser::parse_map(&map)
            }));
        }

        let mut results = vec![];
        for thread in threads {
            results.push(thread.join().unwrap());
        }

        todo!("
            the implementation above is just a placeholder

            the idea is to:
              1. load the source map
              2. split the source map into N chunks, where N is the number of threads
              3. spawn N threads, each of which parses a chunk of the source map
              4. in each thread, iterate over the sources in the chunk and:
                first we need to check if the source is present in the cache, if yes, load the cached tree,
                and check if the hash of the source matches the hash of the cached tree, if yes, return the cached tree,
                otherwise, parse the source and save the tree to the cache
                if the source is not present in the cache, parse the source and save the tree to the cache.
                If the parser failed, return the report immediately and do not continue
              5. join the threads and collect the results
                If any of the threads failed, return the report immediately and do not continue
              6. merge the results into a single forest
              7. return the forest
        ");
    }
}

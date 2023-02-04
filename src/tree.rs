use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use ara_parser::tree::Tree;
use ara_source::source::Source;
use ara_source::source::SourceKind;

use crate::config::Config;
use crate::error::Error;
use crate::hash::ContentHasher;
use crate::serializer::Serializer;
use crate::ARA_CACHED_SOURCE_EXTENSION;
use crate::ARA_DEFINITION_EXTENSION;

pub struct TreeBuilder<'a> {
    config: &'a Config,
    hasher: Box<dyn ContentHasher>,
    serializer: Box<dyn Serializer>,
}

impl<'a> TreeBuilder<'a> {
    pub fn new(
        config: &'a Config,
        hasher: Box<dyn ContentHasher>,
        serializer: Box<dyn Serializer>,
    ) -> Self {
        Self {
            config,
            hasher,
            serializer,
        }
    }

    pub fn build(&self, source_path: &Path) -> Result<(Source, Tree), Error> {
        let source = self.build_source(source_path)?;
        let tree = self.build_tree(&source)?;

        Ok((source, tree))
    }

    fn build_tree(&self, source: &Source) -> Result<Tree, Error> {
        if self.config.cache.is_none() {
            return ara_parser::parser::parse(source).map_err(Error::ParseError);
        }

        let cached_file_path = self.get_cached_file_path(source);

        let tree = self.get_from_cache(source, &cached_file_path).or_else(
            |error| -> Result<Tree, Error> {
                if let Error::DecodeError(_) = error {
                    log::error!(
                        "error while decoding cached file ({}) for source ({}): {}",
                        self.strip_root(&cached_file_path),
                        source.origin.as_ref().unwrap(),
                        error
                    );
                }

                let tree = ara_parser::parser::parse(source).map_err(Error::ParseError)?;
                self.save_to_cache(&cached_file_path, &tree)?;

                Ok(tree)
            },
        )?;

        Ok(tree)
    }

    fn get_from_cache(&self, source: &Source, cached_file_path: &PathBuf) -> Result<Tree, Error> {
        let origin = source.origin.clone().unwrap();
        let definitions = self.serializer.decode(&fs::read(cached_file_path)?)?;

        log::info!(
            "loaded ({}) source from cache ({}).",
            origin,
            self.strip_root(cached_file_path),
        );

        Ok(Tree::new(origin, definitions))
    }

    fn save_to_cache(&self, cached_file_path: &PathBuf, tree: &Tree) -> Result<(), Error> {
        let mut file = File::create(cached_file_path)?;
        let encoded = self.serializer.encode(&tree.definitions)?;
        file.write_all(&encoded)?;

        log::info!(
            "saved ({}) source to cache ({}).",
            tree.source,
            self.strip_root(cached_file_path),
        );

        Ok(())
    }

    fn get_cached_file_path(&self, source: &Source) -> PathBuf {
        let cache_path = self.config.cache.as_ref().unwrap();
        cache_path
            .join(self.hasher.hash(&source.content).to_string())
            .with_extension(ARA_CACHED_SOURCE_EXTENSION)
    }

    fn build_source(&self, source_path: &Path) -> Result<Source, Error> {
        let origin = self.strip_root(source_path);
        let kind = match source_path.extension() {
            Some(extension) if extension == ARA_DEFINITION_EXTENSION => SourceKind::Definition,
            _ => SourceKind::Script,
        };
        let content = fs::read_to_string(source_path)?;

        Ok(Source::new(kind, origin, content))
    }

    fn strip_root(&self, path: &Path) -> String {
        path.strip_prefix(&self.config.root)
            .map(|path| path.to_string_lossy())
            .unwrap()
            .to_string()
    }
}

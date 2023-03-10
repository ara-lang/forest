use bincode::Decode;
use bincode::Encode;
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
use crate::ARA_CACHED_SOURCE_EXTENSION;
use crate::ARA_DEFINITION_EXTENSION;

#[derive(Debug, Hash, Encode, Decode)]
pub struct SignedTree {
    pub signature: u64,
    pub tree: Tree,
}

pub struct TreeBuilder<'a> {
    config: &'a Config,
}

impl<'a> TreeBuilder<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
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
                if let Error::DeserializeError(_) = error {
                    log::error!(
                        "error while deserializing cached file ({}) for source ({}): {}",
                        self.strip_root(&cached_file_path),
                        source.origin.as_ref().unwrap(),
                        error
                    );
                }

                let tree = ara_parser::parser::parse(source).map_err(Error::ParseError)?;
                self.save_to_cache(source, tree, &cached_file_path)
            },
        )?;

        Ok(tree)
    }

    fn get_from_cache(&self, source: &Source, cached_file_path: &PathBuf) -> Result<Tree, Error> {
        let signed_tree = self
            .config
            .serializer
            .deserialize(&fs::read(cached_file_path)?)?;

        let current_signature = self.config.hasher.hash(&source.content);
        if signed_tree.signature != current_signature {
            log::warn!(
                "cache miss due to source change ({}).",
                source.origin.as_ref().unwrap(),
            );

            return Err(Error::CacheMiss);
        }

        log::info!(
            "loaded ({}) parsed source from cache ({}).",
            source.origin.as_ref().unwrap(),
            self.strip_root(cached_file_path),
        );

        Ok(signed_tree.tree)
    }

    fn save_to_cache(
        &self,
        source: &Source,
        tree: Tree,
        cached_file_path: &PathBuf,
    ) -> Result<Tree, Error> {
        let mut file = File::create(cached_file_path)?;
        let signed_tree = SignedTree::new(self.config.hasher.hash(&source.content), tree);

        let serialized = self.config.serializer.serialize(&signed_tree)?;
        file.write_all(&serialized)?;

        log::info!(
            "saved ({}) parsed source to cache ({}).",
            &signed_tree.tree.source,
            self.strip_root(cached_file_path),
        );

        Ok(signed_tree.tree)
    }

    fn get_cached_file_path(&self, source: &Source) -> PathBuf {
        let cache_path = self.config.cache.as_ref().unwrap();
        cache_path
            .join(
                self.config
                    .hasher
                    .hash(source.origin.as_ref().unwrap())
                    .to_string(),
            )
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

impl SignedTree {
    pub fn new(signature: u64, tree: Tree) -> Self {
        Self { signature, tree }
    }
}

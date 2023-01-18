use ara_source::loader::DirectorySourceLoader;
use ara_source::loader::SourceLoader;
use ara_source::SourceMap;

use crate::config::Configuration;
use crate::error::Result;

pub fn load(config: &Configuration) -> Result<SourceMap> {
    let mut paths = vec![];
    paths.push(&config.project.source);
    for definition in &config.definitions {
        paths.push(&definition.path);
    }

    let mut map = SourceMap::new(vec![]);

    let loader = DirectorySourceLoader::new(&config.root);

    for directory in paths {
        let mut source = loader.load(&directory)?;

        map.merge(&mut source);
    }

    Ok(map)
}

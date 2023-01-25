use std::hash::Hasher;

pub struct Config {
    pub root: String,
    pub source: String,
    pub definitions: Vec<String>,
    pub cache: String,
    pub threads: usize,
    pub hasher: Box<dyn Hasher>,
}

impl Config {
    pub fn new<R, S, D, C>(
        root: R,
        source: S,
        definitions: Vec<D>,
        cache: Option<C>,
        threads: Option<usize>,
        hasher: Option<Box<dyn Hasher>>,
    ) -> Self
    where
        R: Into<String>,
        S: Into<String>,
        D: Into<String>,
        C: Into<String>,
    {
        let root = root.into();
        let cache = cache
            .map(|c| c.into())
            .unwrap_or_else(|| format!("{}/.cache", root));

        Self {
            root,
            source: source.into(),
            definitions: definitions.into_iter().map(|d| d.into()).collect(),
            cache,
            threads: threads.unwrap_or_else(num_cpus::get),
            hasher: hasher
                .unwrap_or_else(|| Box::new(std::collections::hash_map::DefaultHasher::new())),
        }
    }
}

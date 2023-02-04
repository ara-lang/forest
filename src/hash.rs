use std::hash::Hasher;

pub trait ContentHasher: Send + Sync {
    fn hash(&self, content: &str) -> u64;
}

pub struct FxHasher;

impl FxHasher {
    pub fn new() -> Self {
        Self
    }
}

impl ContentHasher for FxHasher {
    fn hash(&self, content: &str) -> u64 {
        let mut hasher = rustc_hash::FxHasher::default();
        hasher.write(content.as_bytes());
        hasher.finish()
    }
}

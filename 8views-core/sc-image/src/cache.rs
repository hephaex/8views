use image::DynamicImage;
use lru::LruCache;
use std::num::NonZeroUsize;

const DEFAULT_CACHE_SIZE: usize = 50;

pub struct ImageCache {
    inner: LruCache<usize, DynamicImage>,
}

impl ImageCache {
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity.max(1)).unwrap();
        Self {
            inner: LruCache::new(cap),
        }
    }

    pub fn get(&mut self, index: usize) -> Option<&DynamicImage> {
        self.inner.get(&index)
    }

    pub fn insert(&mut self, index: usize, img: DynamicImage) {
        self.inner.put(index, img);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new(DEFAULT_CACHE_SIZE)
    }
}

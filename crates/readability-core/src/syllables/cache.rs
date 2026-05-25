use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

static SYLLABLE_CACHE: Mutex<Option<LruCache<String, u8>>> = Mutex::new(None);

const CACHE_CAPACITY: usize = 2048;

static CACHE_HITS: AtomicUsize = AtomicUsize::new(0);
static CACHE_MISSES: AtomicUsize = AtomicUsize::new(0);

fn with_cache<F, R>(f: F) -> R
where
    F: FnOnce(&mut LruCache<String, u8>) -> R,
{
    let mut guard = SYLLABLE_CACHE.lock().unwrap();
    let cache = guard.get_or_insert_with(|| {
        LruCache::new(NonZeroUsize::new(CACHE_CAPACITY).unwrap())
    });
    f(cache)
}

pub fn cache_get(word: &str) -> Option<u8> {
    let result = with_cache(|cache| cache.get(word).copied());
    if result.is_some() {
        CACHE_HITS.fetch_add(1, Ordering::Relaxed);
    } else {
        CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
    }
    result
}

pub fn cache_put(word: String, count: u8) {
    with_cache(|cache| {
        cache.put(word, count);
    });
}

pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub len: usize,
    pub capacity: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }
}

pub fn cache_stats() -> CacheStats {
    let len = with_cache(|cache| cache.len());
    CacheStats {
        hits: CACHE_HITS.load(Ordering::Relaxed),
        misses: CACHE_MISSES.load(Ordering::Relaxed),
        len,
        capacity: CACHE_CAPACITY,
    }
}

pub fn reset_stats() {
    CACHE_HITS.store(0, Ordering::Relaxed);
    CACHE_MISSES.store(0, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_put_and_get() {
        cache_put("hello".to_string(), 2);
        assert_eq!(cache_get("hello"), Some(2));
    }

    #[test]
    fn cache_miss() {
        assert_eq!(cache_get("nonexistent_word_xyz"), None);
    }
}

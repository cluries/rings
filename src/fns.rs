use std::collections::HashMap;
use std::hash::Hash;

/// 重试执行函数
pub async fn retry_async<F, Fut, T, E>(
    mut f: F,
    max_attempts: usize,
    delay: std::time::Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts >= max_attempts => return Err(e),
            Err(_) => {
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// 同步重试执行函数
pub fn retry_sync<F, T, E>(
    mut f: F,
    max_attempts: usize,
    delay: std::time::Duration,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match f() {
            Ok(result) => return Ok(result),
            Err(e) if attempts >= max_attempts => return Err(e),
            Err(_) => {
                std::thread::sleep(delay);
            }
        }
    }
}

/// 缓存函数结果
pub struct Cache<K, V> {
    data: HashMap<K, V>,
    max_size: usize,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(max_size: usize) -> Self {
        Self {
            data: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.data.len() >= self.max_size && !self.data.contains_key(&key) {
            // 简单的LRU策略：移除第一个元素
            if let Some(first_key) = self.data.keys().next().cloned() {
                self.data.remove(&first_key);
            }
        }
        self.data.insert(key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key)
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// 批量处理函数
pub async fn batch_process<T, R, F, Fut>(
    items: Vec<T>,
    batch_size: usize,
    processor: F,
) -> Vec<R>
where
    T: Clone,
    F: Fn(Vec<T>) -> Fut,
    Fut: std::future::Future<Output = Vec<R>>,
{
    let mut results = Vec::new();
    
    for chunk in items.chunks(batch_size) {
        let batch_results = processor(chunk.to_vec()).await;
        results.extend(batch_results);
    }
    
    results
}

/// 并发处理函数
pub async fn concurrent_process<T, R, F, Fut>(
    items: Vec<T>,
    max_concurrent: usize,
    processor: F,
) -> Vec<R>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = R> + Send,
{
    use tokio::sync::Semaphore;
    use std::sync::Arc;

    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let processor = Arc::new(processor);
    
    let tasks: Vec<_> = items
        .into_iter()
        .map(|item| {
            let semaphore = semaphore.clone();
            let processor = processor.clone();
            
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                processor(item).await
            })
        })
        .collect();

    let mut results = Vec::new();
    for task in tasks {
        if let Ok(result) = task.await {
            results.push(result);
        }
    }
    
    results
}

/// 防抖函数
pub struct Debouncer<T> {
    delay: std::time::Duration,
    last_call: Option<std::time::Instant>,
    pending_value: Option<T>,
}

impl<T> Debouncer<T> {
    pub fn new(delay: std::time::Duration) -> Self {
        Self {
            delay,
            last_call: None,
            pending_value: None,
        }
    }

    pub fn call(&mut self, value: T) -> bool {
        let now = std::time::Instant::now();
        
        if let Some(last) = self.last_call {
            if now.duration_since(last) < self.delay {
                self.pending_value = Some(value);
                return false;
            }
        }
        
        self.last_call = Some(now);
        self.pending_value = Some(value);
        true
    }

    pub fn take_pending(&mut self) -> Option<T> {
        self.pending_value.take()
    }
}

/// 限流器
pub struct RateLimiter {
    max_requests: usize,
    window: std::time::Duration,
    requests: Vec<std::time::Instant>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: std::time::Duration) -> Self {
        Self {
            max_requests,
            window,
            requests: Vec::new(),
        }
    }

    pub fn try_acquire(&mut self) -> bool {
        let now = std::time::Instant::now();
        
        // 清理过期的请求
        self.requests.retain(|&time| now.duration_since(time) < self.window);
        
        if self.requests.len() < self.max_requests {
            self.requests.push(now);
            true
        } else {
            false
        }
    }

    pub fn remaining(&self) -> usize {
        self.max_requests.saturating_sub(self.requests.len())
    }
}
 
use std::ops::Fn;

/// 对函数进行组合，返回一个新的函数 f(g(x))
pub fn compose<F, G, A, B, C>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(B) -> C,
    G: Fn(A) -> B,
{
    move |x| f(g(x))
}

/// 将函数转换为可缓存的函数
pub fn memoize<F, A, B>(f: F) -> impl Fn(A) -> B
where
    F: Fn(A) -> B,
    A: std::hash::Hash + Eq + Clone,
    B: Clone,
{
    let cache = std::sync::Mutex::new(std::collections::HashMap::<A, B>::new());

    move |x| {
        let mut cache = cache.lock().unwrap();
        if let Some(result) = cache.get(&x) {
            result.clone()
        } else {
            let result = f(x.clone());
            cache.insert(x, result.clone());
            result
        }
    }
}

/// 将函数转换为可重试的函数
pub fn with_retry<F, A, B, E>(f: F, max_retries: u32) -> impl Fn(A) -> Result<B, E>
where
    F: Fn(A) -> Result<B, E>,
    A: Clone,
{
    move |x| {
        let mut retries = 0;
        loop {
            match f(x.clone()) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if retries >= max_retries {
                        return Err(e);
                    }
                    retries += 1;
                },
            }
        }
    }
}

/// 将函数转换为单例函数，确保只会被调用一次
pub fn singleton<F, A, B>(f: F) -> impl Fn(A) -> B
where
    F: Fn(A) -> B,
    A: Clone,
    B: Clone,
{
    let once = std::sync::Once::new();
    let result = std::sync::Mutex::new(None);

    move |x| {
        once.call_once(|| {
            let value = f(x.clone());
            *result.lock().unwrap() = Some(value);
        });
        result.lock().unwrap().as_ref().unwrap().clone()
    }
}


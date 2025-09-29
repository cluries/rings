use crate::erx::{self, ResultBoxedE};
use std::ops::IndexMut;

///
/// 一个不需要严谨的调度系统,不考虑线程安全等
///
pub struct Balanced<T> {
    counter: u128,
    weights: Vec<Weighted<T>>,
    weights_pool: Vec<usize>,
    circle: usize, // Current position in the round-robin cycle
}

#[derive(Debug)]
pub struct InvokedLink {
    pub weight_id: u32,
    pub concurrent_id: u32,
    pub version: u128,
}

pub struct Weighted<T> {
    id: u32,
    weight: u8,
    concurrents: Vec<Concurrent>,
    condition: Box<dyn Fn(&Job) -> bool>,
    invoker: Box<dyn Fn() -> T>,
}

#[derive(Clone, Debug)]
pub struct Job {
    id: i32,
    name: String,
    normal_timeout: u128,
}

#[derive(Clone, Debug)]
pub struct Concurrent {
    id: u32,
    version: u128,
    start: u128,
    end: u128,
}

fn millis() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()
}

// 求最大公约数
pub fn vector_gcd(numbers: &[u8]) -> Option<u8> {
    if numbers.is_empty() {
        return None;
    }
    if numbers.len() == 1 {
        return Some(numbers[0]);
    }

    #[inline]
    fn gcd(mut a: u8, mut b: u8) -> u8 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }

    numbers.iter().fold(numbers[0], |a, &b| gcd(a, b)).into()
}

impl<T> Default for Balanced<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Balanced<T> {
    pub fn new() -> Self {
        Balanced { counter: 0, weights: Vec::new(), weights_pool: Vec::new(), circle: 0 }
    }

    pub fn count(&self) -> u128 {
        self.counter
    }

    pub fn weights(&self) -> &Vec<Weighted<T>> {
        &self.weights
    }

    pub fn add_weight(&mut self, weight: Weighted<T>) -> &mut Self {
        self.weights.push(weight);
        self.rebuild_weight_pool();

        self
    }

    pub fn add_weights(&mut self, weights: Vec<Weighted<T>>) -> &mut Self {
        self.weights.extend(weights);
        self.rebuild_weight_pool();
        self
    }

    pub fn set_weight_val(&mut self, weight_id: u32, weight: u8) -> &mut Self {
        for w in self.weights.iter_mut() {
            if w.id == weight_id {
                w.weight = weight;
                self.rebuild_weight_pool();
                break;
            }
        }

        self
    }

    fn rebuild_weight_pool(&mut self) -> &mut Self {
        let mut weights: Vec<u8> = self.weights.iter().filter(|w| w.weight > 0).map(|w| w.weight).collect();

        if let Some(gcd @ 2..) = vector_gcd(&weights) {
            weights.iter_mut().for_each(|x| *x /= gcd);
        }

        let cap = weights.iter().fold(0, |a, b| a + *b as usize);
        self.weights_pool.resize(cap, 0);

        let mut idx: usize = 0;
        while idx < cap {
            for (index, weight) in weights.iter_mut().enumerate() {
                if *weight > 0 {
                    self.weights_pool[idx] = index;
                    *weight -= 1;
                    idx += 1;
                }
            }
        }

        self.circle = 0;

        // println!("Weights:{:?}", self.weights_pool);

        self
    }

    pub fn balance(&mut self, job: &Job) -> ResultBoxedE<(T, InvokedLink)> {
        fn try_acquire_resource<T>(weight: &mut Weighted<T>, job: &Job, used_millis: u128) -> Option<(T, InvokedLink)> {
            if !(weight.condition)(job) {
                return None;
            }

            match weight.try_using(job.normal_timeout, used_millis) {
                Ok((concurrent_id, version)) => {
                    let result = (weight.invoker)();
                    let weight_id = weight.id;
                    Some((result, InvokedLink { weight_id, concurrent_id, version }))
                },
                Err(ex) => {
                    tracing::error!("{}", ex.message());
                    None
                },
            }
        }

        if self.weights.is_empty() {
            return Err(erx::Erx::boxed("no weights available"));
        }

        if self.weights_pool.is_empty() {
            return Err(erx::Erx::boxed("all weights have zero priority"));
        }

        let used_millis = millis();
        let weights_pool_len = self.weights_pool.len();
        let mut attempts = self.weights.len();

        while attempts > 0 {
            let index = self.weights_pool[self.circle];
            let weight = self.weights.index_mut(index);
            self.circle = (self.circle + 1) % weights_pool_len;

            if let Some(linked) = try_acquire_resource(weight, job, used_millis) {
                return Ok(linked);
            }

            attempts -= 1;
        }

        Err(erx::Erx::boxed("no available resources"))
    }

    pub fn unlock(&mut self, link: &InvokedLink) -> &mut Self {
        for weight in self.weights.iter_mut() {
            if weight.id == link.weight_id {
                weight.unlock(link.concurrent_id, link.version);
            }
        }
        self
    }
}

const DEFAULT_CONCURRENT_TIMEOUT: u128 = 1_000_000_000;

impl<T> Weighted<T> {
    pub fn new(
        id: u32, weight: u8, concurrents: Vec<Concurrent>, condition: Box<dyn Fn(&Job) -> bool>, invoker: Box<dyn Fn() -> T>,
    ) -> Self {
        Self { id, weight, concurrents, condition, invoker }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn weight(&self) -> u8 {
        self.weight
    }

    pub fn get_new_concurrents_id_start(&self) -> u32 {
        let mut min = 0;
        for concurrent in self.concurrents.iter() {
            if concurrent.id > min {
                min = concurrent.id;
            }
        }

        min + 1
    }

    pub fn unlock(&mut self, concurrent_id: u32, version: u128) -> &mut Self {
        for concurrent in self.concurrents.iter_mut() {
            if concurrent.id == concurrent_id {
                concurrent.unlock_versioned(version);
            }
        }

        self
    }

    pub fn add_concurrent(&mut self, concurrent: Concurrent) -> ResultBoxedE<()> {
        for c in self.concurrents.iter() {
            if c.id == concurrent.id {
                return Err(erx::Erx::boxed("concurrent id already exists"));
            }
        }

        self.concurrents.push(concurrent);
        Ok(())
    }

    pub fn remove_concurrent(&mut self, concurrent_id: u32) -> &mut Self {
        self.concurrents.retain(|c| c.id != concurrent_id);
        self
    }

    pub fn clear_concurrent(&mut self) -> &mut Self {
        self.concurrents.clear();
        self
    }

    pub fn concurrents_count(&self) -> usize {
        self.concurrents.len()
    }

    pub fn try_using(&mut self, timeout: u128, used_millis: u128) -> ResultBoxedE<(u32, u128)> {
        let millis = if used_millis == 0 { millis() } else { used_millis };

        for concurrent in self.concurrents.iter_mut() {
            if concurrent.is_busy(millis) {
                continue;
            }

            if timeout == 0 {
                concurrent.reset(DEFAULT_CONCURRENT_TIMEOUT, millis);
            } else {
                concurrent.reset(timeout, millis);
            };

            let r = (concurrent.id, concurrent.version);
            return Ok(r);
        }

        Err(erx::Erx::boxed("all concurrents are busy"))
    }
}

impl Concurrent {
    pub fn new(id: u32) -> Concurrent {
        let start = 0;
        let version = 0;
        let end = 0;
        Concurrent { id, version, start, end }
    }

    pub fn make_concurrents(size: usize, id_start: u32) -> Vec<Concurrent> {
        let mut concurrent = Vec::with_capacity(size);
        for i in 0..(size as u32) {
            concurrent.push(Concurrent::new(id_start + i));
        }

        // println!("==={:?}", concurrent);
        concurrent
    }

    pub fn clear(&mut self) -> &mut Concurrent {
        self.start = 0;
        self.end = 0;
        self
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn version(&self) -> u128 {
        self.version
    }

    pub fn unlock_versioned(&mut self, version: u128) -> &mut Self {
        if self.version == version {
            self.start = 0;
            self.end = 0;
        }

        self
    }

    pub fn unlock(&mut self) -> &mut Concurrent {
        self.start = 0;
        self.end = 0;
        self
    }

    pub fn reset(&mut self, timeout: u128, used_millis: u128) -> &mut Self {
        self.version += 1;
        if used_millis == 0 {
            self.start = millis();
        } else {
            self.start = used_millis;
        }
        self.end = self.start + timeout;
        self
    }

    pub fn is_busy(&self, used_millis: u128) -> bool {
        self.end != 0 && self.end > self.start && if used_millis == 0 { millis() } else { used_millis } < self.end
    }

    pub fn is_idle(&self, used_millis: u128) -> bool {
        !self.is_busy(used_millis)
    }
}

impl Job {
    pub fn new(id: i32, name: &str, normal_timeout: u128) -> Job {
        let name = name.to_string();
        Job { id, name, normal_timeout }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn normal_timeout(&self) -> u128 {
        self.normal_timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct AI {}

    impl AI {
        fn new() -> AI {
            AI {}
        }
    }

    #[test]
    fn test_balanced() {
        let mut bd: Balanced<AI> = Balanced::new();
        // bd.add_weight(
        //     weights_factory(0, 4)
        // ).add_weight(
        //     weights_factory(1, 3)
        // ).add_weight(
        //     weights_factory(2, 2)
        // );

        let weights = vec![weights_factory(0, 40), weights_factory(1, 60), weights_factory(2, 20)];

        bd.add_weights(weights);

        let job = Job::new(1, "test job", 1_000_000);

        for _ in 0..10 {
            let (ai, link) = bd.balance(&job).unwrap();
            bd.unlock(&link);
            println!("==={:?} {:?}", ai, link);
        }
    }

    fn weights_factory(factory: u32, weight: u8) -> Weighted<AI> {
        let concurrents = Concurrent::make_concurrents(8, 0);

        Weighted::new(factory, weight, concurrents, Box::new(|job| -> bool { job.id() >= 0 }), Box::new(|| -> AI { AI::new() }))
    }
}

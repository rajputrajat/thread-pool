use std::{
    sync::{Arc, Mutex},
    thread::{self, sleep, JoinHandle},
    time::Duration,
};

#[derive(Clone)]
pub struct ThreadPool {
    count: usize,
    check_duration: Duration,
    threads_in_use: ThreadCounter,
}

impl ThreadPool {
    pub fn new(count: usize, check_duration: Duration) -> Self {
        Self {
            count,
            check_duration,
            threads_in_use: Arc::new(Mutex::new(0)),
        }
    }

    pub fn spawn<T, F>(&self, f: F) -> JoinHandle<T>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let s = self.clone();
        let jh = thread::spawn(move || {
            let _th_manager = ThreadLifeManager::create_and_wait(s);
            f()
        });
        jh
    }
}

#[cfg(test)]
mod tests {
    use std::{
        thread::{sleep, JoinHandle},
        time::{Duration, Instant},
    };

    use super::ThreadPool;

    const WAIT: usize = 100;

    #[test]
    fn multiple() {
        let th_pool = ThreadPool::new(30, Duration::from_millis(5));
        let thv: Vec<JoinHandle<_>> = (0..100)
            .map(|_| {
                let now = Instant::now();
                th_pool.spawn(move || {
                    sleep(Duration::from_millis(WAIT as u64));
                    Instant::now() - now
                })
            })
            .collect();
        assert_eq!(
            thv.into_iter().fold([0; 4], |mut arr, th| {
                let spent = th.join().unwrap();
                arr[(spent.as_millis() as usize / WAIT) as usize - 1] += 1;
                arr
            }),
            [30, 30, 30, 10]
        );
    }
}

type ThreadCounter = Arc<Mutex<usize>>;

struct ThreadLifeManager(ThreadCounter);
impl ThreadLifeManager {
    fn create_and_wait(thp: ThreadPool) -> Self {
        let manager = Self(thp.threads_in_use);
        loop {
            let cnt = *manager.0.lock().unwrap();
            if cnt >= thp.count {
                sleep(Duration::from(thp.check_duration));
            } else {
                *manager.0.lock().unwrap() += 1;
                break;
            }
        }
        manager
    }
}
impl Drop for ThreadLifeManager {
    fn drop(&mut self) {
        *self.0.lock().unwrap() -= 1;
    }
}

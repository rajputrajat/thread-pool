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
        let th_manager = ThreadLifeManager::create_and_wait(self.clone());
        let jh = thread::spawn(f);
        drop(th_manager);
        jh
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

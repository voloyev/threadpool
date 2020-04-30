use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub struct ThreadPool {
    _handles: Vec<JoinHandle<()>>,
    sender: Sender<Box<dyn FnMut() + Send>>,
}

impl ThreadPool {
    pub fn new(num_threads: u16) -> Self {
        let (sender, receiver) = channel::<Box<dyn FnMut() + Send>>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut _handles: Vec<JoinHandle<()>> = vec![];

        for _ in 0..num_threads {
            let clone = receiver.clone();
            let handle = std::thread::spawn(move || loop {
                let mut work = match clone.lock().unwrap().recv() {
                    Ok(work) => work,
                    Err(_) => break,
                };
                println!("Start");
                work();
                println!("End");
            });

            _handles.push(handle);
        }

        Self { _handles, sender }
    }

    pub fn execute<T: FnMut() + Send + 'static>(&self, work: T) {
        self.sender.send(Box::new(work)).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let pool: ThreadPool = ThreadPool::new(10);
        let foo = || std::thread::sleep(std::time::Duration::from_secs(1));
        pool.execute(foo.clone());
        pool.execute(foo);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    #[test]
    fn addition() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let n = AtomicU32::new(0);
        let nref = Arc::new(n);
        let pool: ThreadPool = ThreadPool::new(10);
        let clone = nref.clone();
        let foo = move || {
            clone.fetch_add(1, Ordering::SeqCst);
        };

        pool.execute(foo.clone());
        pool.execute(foo.clone());
        std::thread::sleep(std::time::Duration::from_secs(1));

        assert_eq!(nref.load(Ordering::SeqCst), 2);
    }
}

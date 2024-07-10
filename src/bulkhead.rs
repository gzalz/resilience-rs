use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::time::Duration;

struct Bulkhead {
    max_concurrent_tasks: usize,
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl Bulkhead {
    fn new(max_concurrent_tasks: usize) -> Self {
        Self {
            max_concurrent_tasks,
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent_tasks)),
        }
    }

    async fn call<F, T, E>(&self, func: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: From<&'static str> + Send + 'static,
    {
        let permit = self.semaphore.acquire().await.unwrap();
        let result = func();
        drop(permit);
        result
    }
}

#[tokio::main]
async fn main() {
    let bulkhead = Arc::new(Bulkhead::new(3));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let bulkhead = bulkhead.clone();
            tokio::spawn(async move {
                let task = move || -> Result<(), &'static str> {
                    // Simulate a task
                    println!("Task {} is running", i);
                    thread::sleep(Duration::from_secs(2));
                    println!("Task {} is done", i);
                    Ok(())
                };

                match bulkhead.call(task).await {
                    Ok(_) => println!("Task {} succeeded", i),
                    Err(err) => println!("Task {} failed with error: {}", i, err),
                }
            })
        })
        .collect();

    futures::future::join_all(handles).await;
}


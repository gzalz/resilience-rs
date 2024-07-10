use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

struct RetryPolicy {
    max_retries: usize,
    delay: Duration,
}

impl RetryPolicy {
    fn new(max_retries: usize, delay: Duration) -> Self {
        Self { max_retries, delay }
    }

    async fn call<F, T, E>(&self, func: Arc<Mutex<Box<F>>>) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E> + Send  + 'static + ?Sized,
        T: Send + 'static,
        E: From<&'static str> + Send + 'static,
    {
        let mut attempts = 0;

        loop {
            let mut func = func.lock().await;
            match (*func)() {
                Ok(result) => return Ok(result),
                Err(err) => {
                    attempts += 1;
                    if attempts >= self.max_retries {
                        return Err(err);
                    } else {
                        sleep(self.delay).await;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
async fn test_retrypolicy() {
    let retry_policy = Arc::new(RetryPolicy::new(3, Duration::from_secs(1))); // 3 retries, 1 second delay

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let retry_policy = retry_policy.clone();
            tokio::spawn(async move {
                let task: Arc<Mutex<Box<dyn FnMut() -> Result<(), &'static str> + Send>>> =
                    Arc::new(Mutex::new(Box::new(move || -> Result<(), &'static str> {
                        // Simulate a task that might fail
                        println!("Task {} is running", i);
                        if i % 2 == 0 {
                            println!("Task {} failed", i);
                            Err("Task failed")
                        } else {
                            println!("Task {} succeeded", i);
                            Ok(())
                        }
                    })));

                let res: Result<_, _> = retry_policy.call(task).await;
                match res {
                    Ok(_) => println!("Task {} succeeded after retries", i),
                    Err(err) => println!("Task {} failed after retries with error: {}", i, err),
                }
            })
        })
        .collect();

    futures::future::join_all(handles).await;
}
}

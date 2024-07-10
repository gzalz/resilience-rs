use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

struct RateLimiter {
    max_requests: usize,
    window: Duration,
    requests: Arc<Mutex<Vec<Instant>>>,
}

impl RateLimiter {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn call<F, T, E>(&self, func: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: From<&'static str> + Send + 'static,
    {
        loop {
            let now = Instant::now();
            let mut requests = self.requests.lock().await;

            // Remove outdated requests
            requests.retain(|&time| now.duration_since(time) < self.window);

            if requests.len() < self.max_requests {
                requests.push(now);
                drop(requests);
                return func();
            }

            drop(requests);
            sleep(Duration::from_millis(100)).await; // Sleep a bit before retrying
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
async fn test_ratelimiter() {
    let rate_limiter = Arc::new(RateLimiter::new(5, Duration::new(1, 0))); // 5 requests per second

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let rate_limiter = rate_limiter.clone();
            tokio::spawn(async move {
                let task = move || -> Result<(), &'static str> {
                    // Simulate a task
                    println!("Task {} is running", i);
                    Ok(())
                };

                match rate_limiter.call(task).await {
                    Ok(_) => println!("Task {} succeeded", i),
                    Err(err) => println!("Task {} failed with error: {}", i, err),
                }
            })
        })
        .collect();

    futures::future::join_all(handles).await;
}
}

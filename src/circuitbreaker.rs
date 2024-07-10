use std::time::{Duration, Instant};

#[derive(Debug)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: usize,
    failure_threshold: usize,
    recovery_timeout: Duration,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    fn new(failure_threshold: usize, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            failure_threshold,
            recovery_timeout,
            last_failure_time: None,
        }
    }

    fn call<F, T, E>(&mut self, func: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: From<&'static str>,
    {
        match self.state {
            CircuitBreakerState::Closed => self.call_closed(func),
            CircuitBreakerState::Open => self.call_open(func),
            CircuitBreakerState::HalfOpen => self.call_half_open(func),
        }
    }

    fn call_closed<F, T, E>(&mut self, func: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: From<&'static str>,
    {
        let result = func();
        if result.is_err() {
            self.failure_count += 1;
            if self.failure_count >= self.failure_threshold {
                self.state = CircuitBreakerState::Open;
                self.last_failure_time = Some(Instant::now());
            }
        } else {
            self.failure_count = 0;
        }
        result
    }

    fn call_open<F, T, E>(&mut self, _func: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: From<&'static str>,
    {
        if let Some(last_failure_time) = self.last_failure_time {
            if last_failure_time.elapsed() >= self.recovery_timeout {
                self.state = CircuitBreakerState::HalfOpen;
                self.call_half_open(_func)
            } else {
                Err("Circuit is open".into())
            }
        } else {
            Err("Circuit is open".into())
        }
    }

    fn call_half_open<F, T, E>(&mut self, func: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: From<&'static str>,
    {
        let result = func();
        if result.is_err() {
            self.state = CircuitBreakerState::Open;
            self.last_failure_time = Some(Instant::now());
        } else {
            self.state = CircuitBreakerState::Closed;
            self.failure_count = 0;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn test_circuitbreaker() {
    let breaker = Arc::new(Mutex::new(CircuitBreaker::new(3, Duration::new(5, 0))));

    let task = || -> Result<(), &'static str> {
        // Simulate a task that might fail
        Err("Task failed")
    };

    for _ in 0..5 {
        let mut breaker = breaker.lock().unwrap();
        match breaker.call(task) {
            Ok(_) => println!("Task succeeded"),
            Err(err) => println!("Task failed with error: {}", err),
        }
    }
    }
}

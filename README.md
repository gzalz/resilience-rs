# `resilience-rs`

This is a Rust library for building resilient applications. It provides a set of functional decorators to help you build applications that are able to withstand failures and recover from them.

## Features

### Bulkhead

The bulkhead pattern is a way to limit the number of concurrent requests that can be made to a service. This can help prevent a service from being overwhelmed by too many requests at once.

### Circuit Breaker

The circuit breaker pattern is a way to handle failures in a service by temporarily stopping requests to that service. This can help prevent cascading failures in your application.

### Rate Limiter

The rate limiter pattern is a way to limit the number of requests that can be made to a service over a given period of time. This can help prevent a service from being overwhelmed by too many requests.

### Retry Policy

The retry pattern is a way to automatically retry requests that have failed. This can help improve the reliability of your application by automatically recovering from transient failures.

### Timeout

The timeout pattern is a way to limit the amount of time that a request can take. This can help prevent your application from hanging indefinitely if a service is slow to respond.

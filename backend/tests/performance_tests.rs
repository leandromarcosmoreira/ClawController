use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
    response::IntoResponse,
};
use tower::ServiceExt;
use serde_json::json;
use std::time::Instant;
use tokio::time::{sleep, Duration};

mod common;
use common::*;

#[tokio::test]
async fn test_concurrent_requests() {
    let app = create_test_app().await;
    let start = Instant::now();
    
    // Test concurrent agent creation
    let handles: Vec<_> = (0..100).map(|i| {
        let app = app.clone();
        tokio::spawn(async move {
            let agent_data = create_test_agent_data();
            app.oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/agents")
                    .header("content-type", "application/json")
                    .body(Body::from(agent_data.to_string()))
                    .unwrap()
            ).await
        })
    }).collect();
    
    for handle in handles {
        let response = handle.await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }
    
    let duration = start.elapsed();
    println!("100 concurrent requests completed in {:?}", duration);
    assert!(duration.as_secs() < 10, "Performance test failed: took too long");
}

#[tokio::test]
async fn test_memory_usage() {
    let app = create_test_app().await;
    
    // Create large number of agents
    for i in 0..1000 {
        let agent_data = create_test_agent_data();
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/agents")
                    .header("content-type", "application/json")
                    .body(Body::from(agent_data.to_string()))
                    .unwrap()
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::CREATED);
    }
    
    // Test memory usage by fetching all agents
    let response = app
        .oneshot(Request::builder().uri("/api/agents").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Verify we can handle the load
    let agents: Vec<Agent> = response.json().await;
    assert_eq!(agents.len(), 1000);
}

#[tokio::test]
async fn test_response_time() {
    let app = create_test_app().await;
    
    // Test response time for various endpoints
    let endpoints = vec![
        "/",
        "/api/agents",
        "/api/tasks",
        "/api/openclaw/status",
        "/api/optimization/status",
    ];
    
    for endpoint in endpoints {
        let start = Instant::now();
        let response = app
            .oneshot(Request::builder().uri(endpoint).body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        let duration = start.elapsed();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(duration.as_millis() < 100, "Endpoint {} took too long: {:?}", endpoint, duration);
    }
}

#[tokio::test]
async fn test_caching_performance() {
    let app = create_test_app().await;
    
    // First request (cache miss)
    let start = Instant::now();
    let response = app
        .oneshot(Request::builder().uri("/api/agents").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let first_request_time = start.elapsed();
    
    // Second request (cache hit)
    let start = Instant::now();
    let response = app
        .oneshot(Request::builder().uri("/api/agents").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let second_request_time = start.elapsed();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Cache hit should be faster
    assert!(second_request_time < first_request_time, "Caching not working properly");
    println!("First request: {:?}, Second request: {:?}", first_request_time, second_request_time);
}

#[tokio::test]
async fn test_database_performance() {
    let app = create_test_app().await;
    
    // Test database write performance
    let start = Instant::now();
    for i in 0..100 {
        let agent_data = create_test_agent_data();
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/agents")
                    .header("content-type", "application/json")
                    .body(Body::from(agent_data.to_string()))
                    .unwrap()
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::CREATED);
    }
    let write_time = start.elapsed();
    
    // Test database read performance
    let start = Instant::now();
    let response = app
        .oneshot(Request::builder().uri("/api/agents").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let read_time = start.elapsed();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    println!("100 writes: {:?}, 1 read: {:?}", write_time, read_time);
    assert!(write_time.as_secs() < 5, "Database write performance too slow");
    assert!(read_time.as_millis() < 100, "Database read performance too slow");
}

#[tokio::test]
async fn test_stress_test() {
    let app = create_test_app().await;
    let start = Instant::now();
    
    // High concurrency stress test
    let handles: Vec<_> = (0..500).map(|i| {
        let app = app.clone();
        tokio::spawn(async move {
            // Mix of different operations
            match i % 4 {
                0 => {
                    // Create agent
                    let agent_data = create_test_agent_data();
                    app.oneshot(
                        Request::builder()
                            .method(Method::POST)
                            .uri("/api/agents")
                            .header("content-type", "application/json")
                            .body(Body::from(agent_data.to_string()))
                            .unwrap()
                    ).await
                }
                1 => {
                    // Get agents
                    app.oneshot(Request::builder().uri("/api/agents").body(Body::empty()).unwrap()).await
                }
                2 => {
                    // Create task
                    let task_data = create_test_task_data();
                    app.oneshot(
                        Request::builder()
                            .method(Method::POST)
                            .uri("/api/tasks")
                            .header("content-type", "application/json")
                            .body(Body::from(task_data.to_string()))
                            .unwrap()
                    ).await
                }
                3 => {
                    // Get tasks
                    app.oneshot(Request::builder().uri("/api/tasks").body(Body::empty()).unwrap()).await
                }
                _ => unreachable!()
            }
        })
    }).collect();
    
    let mut success_count = 0;
    for handle in handles {
        let response = handle.await.unwrap();
        if response.status().is_success() {
            success_count += 1;
        }
    }
    
    let duration = start.elapsed();
    println!("500 mixed operations completed in {:?}, success rate: {}/500", duration, success_count);
    
    assert!(success_count >= 450, "Too many failures in stress test");
    assert!(duration.as_secs() < 30, "Stress test took too long");
}

use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
    response::IntoResponse,
};
use tower::ServiceExt;
use serde_json::json;

mod common;
use common::*;

#[tokio::test]
async fn test_basic_endpoints() {
    let app = create_test_app().await;
    
    // Test root endpoint
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test agents endpoint
    let response = app
        .oneshot(Request::builder().uri("/api/agents").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test tasks endpoint
    let response = app
        .oneshot(Request::builder().uri("/api/tasks").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_agent_crud() {
    let app = create_test_app().await;
    
    // Create agent
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
    
    // Get agent
    let response = app
        .oneshot(Request::builder().uri("/api/agents").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_openclaw_integration() {
    let app = create_test_app().await;
    
    // Test OpenClaw status
    let response = app
        .oneshot(Request::builder().uri("/api/openclaw/status").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test OpenClaw agents
    let response = app
        .oneshot(Request::builder().uri("/api/openclaw/agents").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_security_endpoints() {
    let app = create_test_app().await;
    
    // Test user creation
    let user_data = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123",
        "role": "USER",
        "access_level": "ReadWrite",
        "security_level": "Internal"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/security/users")
                .header("content-type", "application/json")
                .body(Body::from(user_data.to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_optimization_endpoints() {
    let app = create_test_app().await;
    
    // Test optimization status
    let response = app
        .oneshot(Request::builder().uri("/api/optimization/status").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test pool status
    let response = app
        .oneshot(Request::builder().uri("/api/optimization/pool/status").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

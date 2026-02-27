use axum::{
    body::Body,
    http::{Request, StatusCode, Method, header},
    response::IntoResponse,
};
use tower::ServiceExt;
use serde_json::json;
use std::collections::HashMap;

mod common;
use common::*;

#[tokio::test]
async fn test_authentication() {
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
    
    // Test login
    let login_data = json!({
        "username": "testuser",
        "password": "password123"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/security/login")
                .header("content-type", "application/json")
                .body(Body::from(login_data.to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let login_response: serde_json::Value = response.json().await;
    assert!(login_response.get("token").is_some());
}

#[tokio::test]
async fn test_authorization() {
    let app = create_test_app().await;
    
    // Create admin user
    let admin_user = create_test_admin_user();
    let token = create_test_session(&admin_user.id).await.token;
    
    // Test admin access
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/agents")
                .header("authorization", format!("Bearer {}", token))
                .send()
                .await
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test unauthorized access
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/agents/test-agent")
                .send()
                .await
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_input_validation() {
    let app = create_test_app().await;
    
    // Test invalid agent data
    let invalid_agent_data = json!({
        "name": "",  // Empty name should fail validation
        "role": "INVALID_ROLE",  // Invalid role
        "temperature": 2.0,  // Temperature out of range
        "max_tokens": -1000  // Negative tokens
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/agents")
                .header("content-type", "application/json")
                .body(Body::from(invalid_agent_data.to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test invalid task data
    let invalid_task_data = json!({
        "title": "",  // Empty title
        "priority": "INVALID_PRIORITY",  // Invalid priority
        "estimated_hours": -5.0  // Negative hours
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/tasks")
                .header("content-type", "application/json")
                .body(Body::from(invalid_task_data.to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_sql_injection_protection() {
    let app = create_test_app().await;
    
    // Test SQL injection attempt
    let malicious_input = "'; DROP TABLE agents; --";
    
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/agents/{}", malicious_input))
                .send()
                .await
                .unwrap()
        )
        .await
        .unwrap();
    
    // Should return 404 or 400, not 500 (which would indicate SQL error)
    assert!(response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::BAD_REQUEST);
    
    // Verify agents table still exists
    let response = app
        .oneshot(Request::builder().uri("/api/agents").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_rate_limiting() {
    let app = create_test_app().await;
    
    // Test rate limiting by making many rapid requests
    let mut success_count = 0;
    let mut rate_limited_count = 0;
    
    for i in 0..100 {
        let response = app
            .oneshot(Request::builder().uri("/api/agents").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        if response.status() == StatusCode::OK {
            success_count += 1;
        } else if response.status() == StatusCode::TOO_MANY_REQUESTS {
            rate_limited_count += 1;
        }
    }
    
    // Should have some rate limiting after many requests
    assert!(rate_limited_count > 0, "Rate limiting not working");
    println!("Success: {}, Rate limited: {}", success_count, rate_limited_count);
}

#[tokio::test]
async fn test_session_management() {
    let app = create_test_app().await;
    
    // Create user and login
    let user_data = json!({
        "username": "sessiontest",
        "email": "session@example.com",
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
    
    // Login
    let login_data = json!({
        "username": "sessiontest",
        "password": "password123"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/security/login")
                .header("content-type", "application/json")
                .body(Body::from(login_data.to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let login_response: serde_json::Value = response.json().await;
    let token = login_response.get("token").unwrap().as_str().unwrap();
    
    // Test authenticated request
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/agents")
                .header("authorization", format!("Bearer {}", token))
                .send()
                .await
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test invalid token
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/agents")
                .header("authorization", "Bearer invalid_token")
                .send()
                .await
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_audit_logging() {
    let app = create_test_app().await;
    
    // Create user
    let user_data = json!({
        "username": "audituser",
        "email": "audit@example.com",
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
    
    // Check audit log
    let response = app
        .oneshot(Request::builder().uri("/api/security/audit").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let audit_log: Vec<serde_json::Value> = response.json().await;
    assert!(!audit_log.is_empty(), "Audit log should not be empty");
    
    // Check if user creation was logged
    let user_creation_logged = audit_log.iter().any(|entry| {
        entry.get("entity_type") == Some(&json!("user")) &&
        entry.get("action") == Some(&json!("CREATE"))
    });
    
    assert!(user_creation_logged, "User creation should be logged in audit trail");
}

#[tokio::test]
async fn test_permission_enforcement() {
    let app = create_test_app().await;
    
    // Create regular user
    let user_data = json!({
        "username": "regularuser",
        "email": "regular@example.com",
        "password": "password123",
        "role": "USER",
        "access_level": "ReadOnly",
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
    
    // Login as regular user
    let login_data = json!({
        "username": "regularuser",
        "password": "password123"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/security/login")
                .header("content-type", "application/json")
                .body(Body::from(login_data.to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let login_response: serde_json::Value = response.json().await;
    let token = login_response.get("token").unwrap().as_str().unwrap();
    
    // Test read access (should work)
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/agents")
                .header("authorization", format!("Bearer {}", token))
                .send()
                .await
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test write access (should fail)
    let agent_data = create_test_agent_data();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/agents")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(agent_data.to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_security_headers() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    // Check for security headers
    let headers = response.headers();
    
    // These headers should be present for security
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-frame-options"));
    assert!(headers.contains_key("x-xss-protection"));
    
    // Check specific values
    assert_eq!(headers.get("x-content-type-options"), Some(&"nosniff".parse().unwrap()));
    assert_eq!(headers.get("x-frame-options"), Some(&"DENY".parse().unwrap()));
}

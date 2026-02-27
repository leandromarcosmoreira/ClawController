# ClawController Quick Fix Guide

## Current Issues
The backend has compilation errors due to SQLx macros and missing database schema. This guide provides step-by-step fixes.

## Step 1: Fix SQLx Issues

### Fix audit.rs format string error
```rust
// Line 61 - Remove extra placeholder
info!(
    "Audit: {} {} {} {} by {}",
    entity_type,
    action,
    entity_id,
    user_id.unwrap_or("system")
);
```

### Fix audit.rs query syntax errors
```rust
// Line 31-38 - Remove extra "n" prefixes
query!(
    r#"
    INSERT INTO audit_log (
        id, entity_type, entity_id, action, old_values, new_values, user_id, user_role, 
        ip_address, user_agent, session_id, timestamp, success, error_message, 
        risk_score, compliance_flags, metadata
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#
)
```

### Fix audit.rs security_events query
```rust
// Line 89-95 - Remove extra "n" prefixes and fix VALUES count
query!(
    r#"
    INSERT INTO security_events (
        id, event_type, severity, description, source_ip, target_resource, user_id, details, created_at
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
    "#
)
```

### Fix audit.rs violation query
```rust
// Line 290 - Fix format string
report["violations"] = serde_json::to_value(
    violations.iter().map(|v| serde_json::json!({
        "type": format!("{}:{}", v.entity_type, v.action),
        "count": v.count,
    }))
)?;
```

### Fix validation.rs format error
```rust
// Line 521 - Fix format string
return Err(format!("MIME type {} not allowed", mime_type));
```

## Step 2: Create Database Schema

### Create basic schema file
```sql
-- Create basic tables
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    status TEXT DEFAULT 'IDLE',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT DEFAULT 'INBOX',
    priority TEXT DEFAULT 'MEDIUM',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL,
    access_level TEXT NOT NULL,
    security_level TEXT NOT NULL,
    is_active BOOLEAN DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    last_accessed TEXT DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1
);

CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    action TEXT NOT NULL,
    old_values TEXT,
    new_values TEXT,
    user_id TEXT,
    user_role TEXT,
    ip_address TEXT,
    user_agent TEXT,
    session_id TEXT,
    timestamp TEXT NOT NULL,
    success BOOLEAN,
    error_message TEXT,
    risk_score INTEGER,
    compliance_flags TEXT,
    metadata TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS security_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    description TEXT NOT NULL,
    source_ip TEXT,
    target_resource TEXT,
    user_id TEXT,
    details TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS performance_metrics (
    id TEXT PRIMARY KEY,
    metric_name TEXT NOT NULL,
    metric_type TEXT NOT NULL,
    value REAL NOT NULL,
    labels TEXT,
    timestamp TEXT NOT NULL,
    source TEXT NOT NULL,
    entity_id TEXT,
    entity_type TEXT,
    unit TEXT,
    threshold_warning REAL,
    threshold_critical REAL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

## Step 3: Initialize Database

```bash
# Create database and schema
sqlite3 database.db < schema.sql

# Prepare SQLx macros
DATABASE_URL=sqlite:./database.db cargo sqlx prepare

# Build and run
DATABASE_URL=sqlite:./database.db cargo run --bin backend
```

## Step 4: Test Basic Functionality

### Test API endpoints
```bash
# Test basic health check
curl http://localhost:8000/

# Test agent endpoints
curl http://localhost:8000/api/agents

# Test task endpoints
curl http://localhost:8000/api/tasks

# Test OpenClaw integration
curl http://localhost:8000/api/openclaw/status
curl http://localhost:8000/api/openclaw/agents
```

## Step 5: Verify Advanced Features

### Test optimization endpoints
```bash
curl http://localhost:8000/api/optimization/status
curl -X POST http://localhost:8000/api/optimization/cache/warm \
  -H "Content-Type: application/json" \
  -d '{"agent_ids": ["agent_1", "agent_2"]}'
```

### Test collaboration features
```bash
curl http://localhost:8000/api/collaboration/status
curl -X POST http://localhost:8000/api/collaboration/teams \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Team", "member_ids": ["user_1", "user_2"]}'
```

### Test security features
```bash
curl -X POST http://localhost:8000/api/security/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password123"}'
```

## Step 6: Performance Testing

### Load testing with curl
```bash
# Test concurrent requests
for i in {1..10}; do
  curl -s http://localhost:8000/api/agents &
done
wait

# Test performance under load
ab -n 100 -c 10 http://localhost:8000/api/agents
```

## Expected Results

After fixes, you should see:
- ✅ Successful compilation
- ✅ Database schema created
- ✅ API endpoints responding
- ✅ OpenClaw integration working
- ✅ Performance monitoring active
- ✅ Security features functional

## Troubleshooting

### If SQLx errors persist:
1. Check database file exists: `ls -la database.db`
2. Verify SQLx config: `cat .sqlx/config.json`
3. Re-run prepare: `DATABASE_URL=sqlite:./database.db cargo sqlx prepare`

### If API doesn't respond:
1. Check server logs for errors
2. Verify database connection
3. Test with `curl -v` for detailed output

### If performance issues:
1. Check metrics endpoint: `/api/optimization/status`
2. Monitor resource usage
3. Review agent configuration

## Next Steps

Once basic functionality is working:
1. Implement enhanced OpenClaw features
2. Add real-time collaboration
3. Implement advanced monitoring
4. Add user experience improvements

# Final SQLx Fixes Required

## Remaining Issues to Fix

### 1. Security.rs Parameter Bindings

#### Fix User Query (Line 702)
```rust
// Current (broken):
let user = query_as!(
    User,
    "SELECT * FROM users WHERE id = ? AND is_active = 1"
)

// Fixed:
let user = query_as!(
    User,
    "SELECT * FROM users WHERE id = ? AND is_active = 1",
    user_id
)
```

#### Fix Session Token Query (Line 717)
```rust
// Current (broken):
query!(
    "UPDATE sessions SET is_active = 0 WHERE token = ?"
)

// Fixed:
query!(
    "UPDATE sessions SET is_active = 0 WHERE token = ?",
    token
)
```

#### Fix Session User ID Query (Line 728)
```rust
// Current (broken):
query!(
    "UPDATE sessions SET is_active = 0 WHERE user_id = ?"
)

// Fixed:
query!(
    "UPDATE sessions SET is_active = 0 WHERE user_id = ?",
    user_id
)
```

#### Fix Security Events INSERT (Line 798)
```rust
// Current (broken):
query!(
    r#"
    INSERT INTO security_events (
n                id, event_type, severity, description, source_ip, target_resource, user_id, details, created_at
n            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
n            "#
)

// Fixed:
query!(
    r#"
    INSERT INTO security_events (
                id, event_type, severity, description, source_ip, target_resource, user_id, details, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#
)
```

### 2. Audit.rs Parameter Bindings

#### Fix Audit Log INSERT (Line 31)
```rust
// Current (broken):
query!(
    r#"
    INSERT INTO audit_log (
                id, entity_type, entity_id, action, old_values, new_values, user_id, user_role, 
                ip_address, user_agent, session_id, timestamp, success, error_message, 
                risk_score, compliance_flags, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
)

// Fixed:
query!(
    r#"
    INSERT INTO audit_log (
                id, entity_type, entity_id, action, old_values, new_values, user_id, user_role, 
                ip_address, user_agent, session_id, timestamp, success, error_message, 
                risk_score, compliance_flags, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
)
```

#### Fix Security Events INSERT (Line 89)
```rust
// Current (broken):
query!(
    r#"
    INSERT INTO security_events (
n                id, event_type, severity, description, source_ip, target_resource, user_id, details, created_at
n            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
n            "#
)

// Fixed:
query!(
    r#"
    INSERT INTO security_events (
                id, event_type, severity, description, source_ip, target_resource, user_id, details, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#
)
```

#### Fix Audit Log DELETE (Line 229)
```rust
// Current (broken):
let result = query!(
    "DELETE FROM audit_log WHERE timestamp < ?"
)

// Fixed:
let result = query!(
    "DELETE FROM audit_log WHERE timestamp < ?",
    cutoff_date
)
```

#### Fix Security Events DELETE (Line 245)
```rust
// Current (broken):
let result = query!(
    "DELETE FROM security_events WHERE created_at < ?"
)

// Fixed:
let result = query!(
    "DELETE FROM security_events WHERE created_at < ?",
    cutoff_date
)
```

### 3. Format String Fix (Line 61)
```rust
// Current (broken):
info!(
    "Audit: {} {} {} {} {} by {}",
    entity_type,
    action,
    entity_id,
    user_id.unwrap_or("system")
)

// Fixed:
info!(
    "Audit: {} {} {} {} by {}",
    entity_type,
    action,
    entity_id,
    user_id.unwrap_or("system")
)
```

## Quick Fix Commands

### Execute All Fixes
```bash
cd /home/lemoreira/git/integracoes/ClawController/backend

# Fix security.rs parameter bindings
sed -i 's/let user = query_as!(\n                User,\n                "SELECT \* FROM users WHERE id = \? AND is_active = 1"\n                user_id\n            )/' src/security.rs

sed -i 's/query!(\n            "UPDATE sessions SET is_active = 0 WHERE token = \?"/\n            token\n        )/' src/security.rs

sed -i 's/query!(\n            "UPDATE sessions SET is_active = 0 WHERE user_id = \?"/\n            user_id\n        )/' src/security.rs

sed -i 's/n                id, event_type, severity, description, source_ip, target_resource, user_id, details, created_at/n                id, event_type, severity, description, source_ip, target_resource, user_id, details, created_at/n            ) VALUES \?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)/n            ) VALUES \?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)/n            "#
        )/' src/security.rs

# Fix audit.rs parameter bindings
sed -i 's/r#"\n    INSERT INTO audit_log (\n                id, entity_type, entity_id, action, old_values, new_values, user_id, user_role, \n                ip_address, user_agent, session_id, timestamp, success, error_message, \n                risk_score, compliance_flags, metadata\n            ) VALUES \?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)/\n            "#
        )/' src/audit.rs

sed -i 's/r#"\n    INSERT INTO security_events (\n                id, event_type, severity, description, source_ip, target_resource, user_id, details, created_at\n            ) VALUES \?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)/\n            ) VALUES \?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)/\n            "#
        )/' src/audit.rs

sed -i 's/DELETE FROM audit_log WHERE timestamp < \?/DELETE FROM audit_log WHERE timestamp < ?/' src/audit.rs

sed -i 's/DELETE FROM security_events WHERE created_at < \?/DELETE FROM security_events WHERE created_at < ?/' src/audit.rs

# Fix format string
sed -i 's/"Audit: {} {} {} {} {} by {}"/"Audit: {} {} {} {} by {}"/' src/audit.rs

# Run SQLx prepare
DATABASE_URL=sqlite:./database.db cargo sqlx prepare
```

## Expected Result
After these fixes, the compilation should succeed and the backend should be ready for full testing of all OpenClaw agent functionalities.

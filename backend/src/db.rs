use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use std::env;
use std::str::FromStr;
use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn, error};

pub async fn setup_db() -> Result<SqlitePool> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:../data/mission_control.db".to_string());
    
    let options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .foreign_keys(true) // Enable foreign key constraints
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .connect_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect_with(options)
        .await?;

    info!("Database connected successfully");

    // Enable WAL mode and other optimizations
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    
    sqlx::query("PRAGMA synchronous=NORMAL")
        .execute(&pool)
        .await?;
    
    sqlx::query("PRAGMA cache_size=10000")
        .execute(&pool)
        .await?;
    
    sqlx::query("PRAGMA temp_store=MEMORY")
        .execute(&pool)
        .await?;

    // Create tables with enhanced schema
    create_core_tables(&pool).await?;
    create_security_tables(&pool).await?;
    create_audit_tables(&pool).await?;
    create_monitoring_tables(&pool).await?;
    create_indexes(&pool).await?;
    create_triggers(&pool).await?;
    
    info!("Database schema initialized successfully");
        
    Ok(pool)
}

async fn create_core_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Enhanced agents table with security and audit fields
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL CHECK(length(name) >= 1 AND length(name) <= 255),
            role TEXT NOT NULL CHECK(role IN ('LEAD', 'INT', 'SPC', 'ADMIN', 'AUDITOR', 'OBSERVER')),
            description TEXT CHECK(description IS NULL OR length(description) <= 1000),
            avatar TEXT,
            status TEXT NOT NULL DEFAULT 'IDLE' CHECK(status IN ('WORKING', 'IDLE', 'STANDBY', 'OFFLINE', 'MAINTENANCE', 'SUSPENDED', 'ERROR')),
            workspace TEXT,
            agent_dir TEXT,
            token TEXT CHECK(token IS NULL OR length(token) >= 32),
            primary_model TEXT,
            fallback_model TEXT,
            current_model TEXT,
            model_failure_count INTEGER DEFAULT 0 CHECK(model_failure_count >= 0 AND model_failure_count <= 100),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            created_by TEXT,
            last_active_at DATETIME,
            version INTEGER DEFAULT 1 CHECK(version >= 1),
            is_active BOOLEAN DEFAULT 1,
            security_level TEXT DEFAULT 'INTERNAL' CHECK(security_level IN ('PUBLIC', 'INTERNAL', 'CONFIDENTIAL', 'RESTRICTED', 'SECRET')),
            access_level TEXT DEFAULT 'READ_WRITE' CHECK(access_level IN ('READ_ONLY', 'READ_WRITE', 'ADMIN', 'SUPER_ADMIN')),
            -- OpenClaw Advanced Configuration
            image_model TEXT,
            sandbox_mode TEXT CHECK(sandbox_mode IN ('OFF', 'ON', 'DOCKER')),
            thinking_default TEXT CHECK(thinking_default IN ('OFF', 'MINIMAL', 'LOW', 'MEDIUM', 'HIGH', 'XHIGH')),
            verbose_default TEXT CHECK(verbose_default IN ('OFF', 'ON', 'FULL')),
            max_concurrent INTEGER CHECK(max_concurrent >= 1 AND max_concurrent <= 10),
            timeout_seconds INTEGER CHECK(timeout_seconds >= 30 AND timeout_seconds <= 3600),
            context_tokens INTEGER CHECK(context_tokens >= 1000 AND context_tokens <= 128000),
            skills TEXT, -- JSON array
            tools_config TEXT, -- JSON object
            memory_search_config TEXT, -- JSON object
            heartbeat_enabled BOOLEAN DEFAULT 0,
            subagents_enabled BOOLEAN DEFAULT 0,
            human_delay_enabled BOOLEAN DEFAULT 0,
            block_streaming_enabled BOOLEAN DEFAULT 0,
            context_pruning_enabled BOOLEAN DEFAULT 0,
            openclaw_config_hash TEXT, -- For sync tracking
            -- Enhanced fields
            tags TEXT, -- JSON array
            metadata TEXT, -- JSON object
            performance_metrics TEXT, -- JSON object
            health_score REAL CHECK(health_score >= 0.0 AND health_score <= 100.0),
            last_health_check DATETIME,
            -- Audit fields
            created_ip TEXT,
            modified_by TEXT,
            modified_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            deleted_at DATETIME,
            deleted_by TEXT,
            is_deleted BOOLEAN DEFAULT 0
        );
        "#
    )
    .execute(pool)
    .await?;
    // Enhanced tasks table with comprehensive tracking
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL CHECK(length(title) >= 1 AND length(title) <= 500),
            description TEXT CHECK(description IS NULL OR length(description) <= 5000),
            status TEXT NOT NULL DEFAULT 'INBOX' CHECK(status IN ('INBOX', 'ASSIGNED', 'IN_PROGRESS', 'REVIEW', 'DONE', 'BLOCKED', 'CANCELLED', 'ARCHIVED')),
            priority TEXT NOT NULL DEFAULT 'NORMAL' CHECK(priority IN ('LOW', 'NORMAL', 'HIGH', 'URGENT', 'CRITICAL')),
            tags TEXT, -- JSON array
            assignee_id TEXT,
            reviewer TEXT,
            reviewer_id TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            due_at DATETIME,
            completed_at DATETIME,
            created_by TEXT NOT NULL,
            estimated_hours REAL CHECK(estimated_hours > 0),
            actual_hours REAL CHECK(actual_hours >= 0),
            complexity_score INTEGER CHECK(complexity_score >= 1 AND complexity_score <= 10),
            risk_level TEXT CHECK(risk_level IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
            dependencies TEXT, -- JSON array
            deliverables TEXT, -- JSON array
            metadata TEXT, -- JSON object
            version INTEGER DEFAULT 1 CHECK(version >= 1),
            is_template BOOLEAN DEFAULT 0,
            template_usage_count INTEGER DEFAULT 0 CHECK(template_usage_count >= 0),
            -- Audit fields
            created_ip TEXT,
            modified_by TEXT,
            deleted_at DATETIME,
            deleted_by TEXT,
            is_deleted BOOLEAN DEFAULT 0,
            -- Constraints
            FOREIGN KEY(assignee_id) REFERENCES agents(id) ON DELETE SET NULL,
            FOREIGN KEY(reviewer_id) REFERENCES agents(id) ON DELETE SET NULL
        );
        "#
    )
    .execute(pool)
    .await?;

    // Enhanced comments with threading and reactions
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS comments (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            content TEXT NOT NULL CHECK(length(content) >= 1 AND length(content) <= 10000),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            is_edited BOOLEAN DEFAULT 0,
            parent_id TEXT, -- For threaded comments
            mentions TEXT, -- JSON array
            attachments TEXT, -- JSON array
            reaction_count INTEGER DEFAULT 0 CHECK(reaction_count >= 0),
            is_deleted BOOLEAN DEFAULT 0,
            deleted_at DATETIME,
            deleted_by TEXT,
            -- Constraints
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE,
            FOREIGN KEY(parent_id) REFERENCES comments(id) ON DELETE CASCADE
        );
        "#
    )
    .execute(pool)
    .await?;

    // Enhanced announcements with targeting
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS announcements (
            id TEXT PRIMARY KEY,
            title TEXT,
            message TEXT NOT NULL CHECK(length(message) >= 1),
            priority TEXT NOT NULL DEFAULT 'NORMAL' CHECK(priority IN ('LOW', 'NORMAL', 'HIGH', 'URGENT', 'CRITICAL')),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            created_by TEXT NOT NULL,
            expires_at DATETIME,
            target_audience TEXT, -- JSON array of roles/ids
            is_active BOOLEAN DEFAULT 1,
            view_count INTEGER DEFAULT 0 CHECK(view_count >= 0),
            -- Constraints
            FOREIGN KEY(created_by) REFERENCES agents(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    // Enhanced deliverables with versioning
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS deliverables (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            title TEXT NOT NULL CHECK(length(title) >= 1 AND length(title) <= 500),
            description TEXT,
            status TEXT NOT NULL DEFAULT 'PENDING' CHECK(status IN ('PENDING', 'IN_PROGRESS', 'COMPLETED', 'FAILED', 'CANCELLED')),
            file_path TEXT,
            file_size INTEGER CHECK(file_size >= 0),
            file_hash TEXT,
            mime_type TEXT,
            version INTEGER DEFAULT 1 CHECK(version >= 1),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            completed_at DATETIME,
            completed_by TEXT,
            -- Constraints
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY(completed_by) REFERENCES agents(id) ON DELETE SET NULL
        );
        "#
    )
    .execute(pool)
    .await?;

    // Enhanced recurring tasks with advanced scheduling
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS recurring_tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL CHECK(length(title) >= 1 AND length(title) <= 500),
            description TEXT CHECK(description IS NULL OR length(description) <= 5000),
            assignee_id TEXT,
            schedule_type TEXT NOT NULL CHECK(schedule_type IN ('DAILY', 'WEEKLY', 'MONTHLY', 'YEARLY', 'CUSTOM')),
            schedule_value TEXT,
            schedule_time TEXT NOT NULL,
            schedule_timezone TEXT DEFAULT 'UTC',
            last_run DATETIME,
            next_run DATETIME NOT NULL,
            is_active BOOLEAN DEFAULT 1,
            max_runs INTEGER,
            run_count INTEGER DEFAULT 0 CHECK(run_count >= 0),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            created_by TEXT NOT NULL,
            -- Constraints
            FOREIGN KEY(assignee_id) REFERENCES agents(id) ON DELETE SET NULL,
            FOREIGN KEY(created_by) REFERENCES agents(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_security_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Users table with comprehensive security
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE CHECK(length(username) >= 3 AND length(username) <= 50),
            email TEXT NOT NULL UNIQUE CHECK(email LIKE '%@%'),
            password_hash TEXT NOT NULL CHECK(length(password_hash) >= 60),
            role TEXT NOT NULL DEFAULT 'USER' CHECK(role IN ('SUPER_ADMIN', 'ADMIN', 'USER', 'READ_ONLY')),
            is_active BOOLEAN DEFAULT 1,
            last_login DATETIME,
            failed_login_attempts INTEGER DEFAULT 0 CHECK(failed_login_attempts >= 0 AND failed_login_attempts <= 10),
            locked_until DATETIME,
            security_level TEXT DEFAULT 'INTERNAL' CHECK(security_level IN ('PUBLIC', 'INTERNAL', 'CONFIDENTIAL', 'RESTRICTED', 'SECRET')),
            access_level TEXT DEFAULT 'READ_ONLY' CHECK(access_level IN ('READ_ONLY', 'READ_WRITE', 'ADMIN', 'SUPER_ADMIN')),
            permissions TEXT, -- JSON array
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_password_change DATETIME DEFAULT CURRENT_TIMESTAMP,
            two_factor_enabled BOOLEAN DEFAULT 0,
            two_factor_secret TEXT,
            email_verified BOOLEAN DEFAULT 0,
            phone_verified BOOLEAN DEFAULT 0,
            profile_picture TEXT,
            timezone TEXT DEFAULT 'UTC',
            language TEXT DEFAULT 'en'
        );
        "#
    )
    .execute(pool)
    .await?;

    // Sessions table with enhanced tracking
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            token TEXT NOT NULL UNIQUE CHECK(length(token) >= 128),
            expires_at DATETIME NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_accessed DATETIME DEFAULT CURRENT_TIMESTAMP,
            ip_address TEXT NOT NULL,
            user_agent TEXT,
            is_active BOOLEAN DEFAULT 1,
            device_fingerprint TEXT,
            login_method TEXT DEFAULT 'password',
            two_factor_verified BOOLEAN DEFAULT 0,
            -- Constraints
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#
    )
    .execute(pool)
    .await?;

    // Permissions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS permissions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            resource TEXT NOT NULL,
            action TEXT NOT NULL,
            conditions TEXT, -- JSON object
            is_system BOOLEAN DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(pool)
    .await?;

    // Roles table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS roles (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            permissions TEXT, -- JSON array
            is_system BOOLEAN DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(pool)
    .await?;

    // User roles junction table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_roles (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            role_id TEXT NOT NULL,
            assigned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            assigned_by TEXT,
            expires_at DATETIME,
            -- Constraints
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY(role_id) REFERENCES roles(id) ON DELETE CASCADE,
            UNIQUE(user_id, role_id)
        );
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_audit_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Comprehensive audit log
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_log (
            id TEXT PRIMARY KEY,
            entity_type TEXT NOT NULL CHECK(entity_type IN ('agent', 'task', 'user', 'session', 'permission', 'role', 'system')),
            entity_id TEXT NOT NULL,
            action TEXT NOT NULL CHECK(action IN ('create', 'update', 'delete', 'access', 'login', 'logout', 'view', 'export', 'import')),
            old_values TEXT, -- JSON object
            new_values TEXT, -- JSON object
            user_id TEXT,
            user_role TEXT,
            ip_address TEXT,
            user_agent TEXT,
            session_id TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            error_message TEXT,
            risk_score INTEGER CHECK(risk_score >= 0 AND risk_score <= 100),
            compliance_flags TEXT, -- JSON array
            metadata TEXT -- JSON object
        );
        "#
    )
    .execute(pool)
    .await?;

    // Security events tracking
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS security_events (
            id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL CHECK(event_type IN ('login_failure', 'unauthorized_access', 'privilege_escalation', 'data_breach', 'suspicious_activity', 'malware_detected')),
            severity TEXT NOT NULL CHECK(severity IN ('low', 'medium', 'high', 'critical')),
            description TEXT NOT NULL,
            source_ip TEXT,
            target_resource TEXT,
            user_id TEXT,
            details TEXT, -- JSON object
            resolved BOOLEAN DEFAULT 0,
            resolved_at DATETIME,
            resolved_by TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            -- Constraints
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE SET NULL,
            FOREIGN KEY(resolved_by) REFERENCES users(id) ON DELETE SET NULL
        );
        "#
    )
    .execute(pool)
    .await?;

    // Activity log (legacy compatibility)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS activity_log (
            id TEXT PRIMARY KEY,
            activity_type TEXT NOT NULL,
            agent_id TEXT,
            task_id TEXT,
            description TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            -- Constraints
            FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE SET NULL,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE SET NULL
        );
        "#
    )
    .execute(pool)
    .await?;

    // Task activity (legacy compatibility)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS task_activity (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            agent_id TEXT,
            message TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            -- Constraints
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE
        );
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_monitoring_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // System configuration
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS system_configuration (
            id TEXT PRIMARY KEY,
            key TEXT NOT NULL UNIQUE,
            value TEXT NOT NULL,
            data_type TEXT NOT NULL CHECK(data_type IN ('string', 'number', 'boolean', 'json')),
            description TEXT,
            category TEXT NOT NULL,
            is_sensitive BOOLEAN DEFAULT 0,
            requires_restart BOOLEAN DEFAULT 0,
            validation_rules TEXT, -- JSON object
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_by TEXT NOT NULL,
            version INTEGER DEFAULT 1 CHECK(version >= 1),
            -- Constraints
            FOREIGN KEY(updated_by) REFERENCES users(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    // Performance metrics
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS performance_metrics (
            id TEXT PRIMARY KEY,
            metric_name TEXT NOT NULL,
            metric_type TEXT NOT NULL CHECK(metric_type IN ('counter', 'gauge', 'histogram', 'timer')),
            value REAL NOT NULL,
            labels TEXT, -- JSON object
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            source TEXT NOT NULL CHECK(source IN ('agent', 'system', 'api', 'database', 'network')),
            entity_id TEXT,
            entity_type TEXT,
            unit TEXT,
            threshold_warning REAL,
            threshold_critical REAL,
            -- Indexes for time-series queries
            CHECK(timestamp IS NOT NULL)
        );
        "#
    )
    .execute(pool)
    .await?;

    // Backup records
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS backup_records (
            id TEXT PRIMARY KEY,
            backup_type TEXT NOT NULL CHECK(backup_type IN ('full', 'incremental', 'differential')),
            location TEXT NOT NULL,
            size_bytes INTEGER NOT NULL CHECK(size_bytes > 0),
            status TEXT NOT NULL DEFAULT 'in_progress' CHECK(status IN ('in_progress', 'completed', 'failed', 'cancelled')),
            started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            completed_at DATETIME,
            error_message TEXT,
            checksum TEXT,
            retention_days INTEGER DEFAULT 30 CHECK(retention_days > 0),
            created_by TEXT NOT NULL,
            -- Constraints
            FOREIGN KEY(created_by) REFERENCES users(id)
        );
        "#
    )
    .execute(pool)
    .await?;

    // OpenClaw configuration snapshots (enhanced)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS openclaw_config_snapshots (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            config_hash TEXT NOT NULL,
            raw_config TEXT NOT NULL, -- Full OpenClaw config JSON
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            is_active BOOLEAN DEFAULT 1,
            backup_type TEXT DEFAULT 'manual',
            created_by TEXT,
            -- Constraints
            FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE,
            FOREIGN KEY(created_by) REFERENCES users(id) ON DELETE SET NULL
        );
        "#
    )
    .execute(pool)
    .await?;

    // Agent parameter history (enhanced)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_parameter_history (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            parameter_name TEXT NOT NULL,
            old_value TEXT,
            new_value TEXT,
            changed_by TEXT, -- 'sync' or 'user'
            changed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            change_reason TEXT,
            -- Constraints
            FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE
        );
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_indexes(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Performance indexes
    let indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status, is_active)",
        "CREATE INDEX IF NOT EXISTS idx_agents_role ON agents(role, security_level)",
        "CREATE INDEX IF NOT EXISTS idx_agents_updated ON agents(updated_at)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status, priority)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_assignee ON tasks(assignee_id, due_at)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_created ON tasks(created_at)",
        "CREATE INDEX IF NOT EXISTS idx_comments_task ON comments(task_id, created_at)",
        "CREATE INDEX IF NOT EXISTS idx_comments_agent ON comments(agent_id, created_at)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id, is_active)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at)",
        "CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_log(entity_type, entity_id)",
        "CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_log(user_id, timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_security_events_created ON security_events(created_at)",
        "CREATE INDEX IF NOT EXISTS idx_security_events_severity ON security_events(severity)",
        "CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON performance_metrics(timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_metrics_name_source ON performance_metrics(metric_name, source)",
        "CREATE INDEX IF NOT EXISTS idx_backups_created ON backup_records(started_at)",
        "CREATE INDEX IF NOT EXISTS idx_config_snapshots_agent ON openclaw_config_snapshots(agent_id, is_active)",
        "CREATE INDEX IF NOT EXISTS idx_param_history_agent ON agent_parameter_history(agent_id, changed_at)",
    ];

    for index in indexes {
        sqlx::query(index).execute(pool).await?;
    }

    Ok(())
}

async fn create_triggers(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Audit triggers
    let triggers = vec![
        // Agent audit trigger
        r#"
        CREATE TRIGGER IF NOT EXISTS agents_audit_insert
        AFTER INSERT ON agents
        BEGIN
            INSERT INTO audit_log (
                entity_type, entity_id, action, new_values, user_id, timestamp, success
            ) VALUES (
                'agent', NEW.id, 'create', json_extract(NEW.*, '$'),
                COALESCE(NEW.created_by, 'system'), CURRENT_TIMESTAMP, 1
            );
        END;
        "#,
        r#"
        CREATE TRIGGER IF NOT EXISTS agents_audit_update
        AFTER UPDATE ON agents
        BEGIN
            INSERT INTO audit_log (
                entity_type, entity_id, action, old_values, new_values, user_id, timestamp, success
            ) VALUES (
                'agent', NEW.id, 'update', json_extract(OLD.*, '$'), json_extract(NEW.*, '$'),
                COALESCE(NEW.modified_by, 'system'), CURRENT_TIMESTAMP, 1
            );
        END;
        "#,
        // Task audit trigger
        r#"
        CREATE TRIGGER IF NOT EXISTS tasks_audit_insert
        AFTER INSERT ON tasks
        BEGIN
            INSERT INTO audit_log (
                entity_type, entity_id, action, new_values, user_id, timestamp, success
            ) VALUES (
                'task', NEW.id, 'create', json_extract(NEW.*, '$'),
                NEW.created_by, CURRENT_TIMESTAMP, 1
            );
        END;
        "#,
        // Update timestamp triggers
        "CREATE TRIGGER IF NOT EXISTS agents_update_timestamp AFTER UPDATE ON agents BEGIN UPDATE agents SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;",
        "CREATE TRIGGER IF NOT EXISTS tasks_update_timestamp AFTER UPDATE ON tasks BEGIN UPDATE tasks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;",
        "CREATE TRIGGER IF NOT EXISTS comments_update_timestamp AFTER UPDATE ON comments BEGIN UPDATE comments SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;",
        "CREATE TRIGGER IF NOT EXISTS sessions_update_timestamp AFTER UPDATE ON sessions BEGIN UPDATE sessions SET last_accessed = CURRENT_TIMESTAMP WHERE id = NEW.id; END;",
    ];

    for trigger in triggers {
        sqlx::query(trigger).execute(pool).await?;
    }

    Ok(())
}

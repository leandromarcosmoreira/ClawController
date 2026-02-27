-- Initial database schema for ClawController
-- This migration creates all necessary tables for the application

-- Agents table
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    status TEXT DEFAULT 'IDLE',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1,
    is_deleted BOOLEAN DEFAULT 0,
    created_by TEXT,
    description TEXT,
    avatar TEXT,
    workspace TEXT,
    token TEXT,
    primary_model TEXT,
    fallback_model TEXT,
    current_model TEXT,
    model_failure_count INTEGER DEFAULT 0,
    skills TEXT,
    max_concurrent INTEGER,
    max_memory_mb INTEGER,
    max_execution_time_minutes INTEGER,
    thinking_default INTEGER,
    temperature REAL,
    max_tokens INTEGER,
    security_level TEXT,
    access_level TEXT,
    sandbox_mode TEXT
);

-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT DEFAULT 'INBOX',
    priority TEXT DEFAULT 'MEDIUM',
    tags TEXT,
    assignee_id TEXT,
    reviewer TEXT,
    reviewer_id TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    due_at TEXT,
    estimated_hours REAL,
    actual_hours REAL,
    complexity_score INTEGER,
    dependencies TEXT,
    deliverables TEXT,
    is_deleted BOOLEAN DEFAULT 0,
    created_by TEXT
);

-- Users table
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
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    profile_picture TEXT,
    timezone TEXT,
    preferences TEXT,
    last_login TEXT,
    failed_login_attempts INTEGER DEFAULT 0,
    locked_until TEXT
);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    last_accessed TEXT DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1,
    ip_address TEXT,
    user_agent TEXT,
    device_fingerprint TEXT
);

-- Audit log table
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

-- Security events table
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

-- Performance metrics table
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

-- Comments table
CREATE TABLE IF NOT EXISTS comments (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    is_deleted BOOLEAN DEFAULT 0,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- Announcements table
CREATE TABLE IF NOT EXISTS announcements (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1,
    created_by TEXT
);

-- Activity table
CREATE TABLE IF NOT EXISTS activity (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    action TEXT NOT NULL,
    user_id TEXT,
    details TEXT,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Task activity table
CREATE TABLE IF NOT EXISTS task_activity (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    user_id TEXT,
    action TEXT NOT NULL,
    details TEXT,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- Deliverables table
CREATE TABLE IF NOT EXISTS deliverables (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT DEFAULT 'PENDING',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- Recurring tasks table
CREATE TABLE IF NOT EXISTS recurring_tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    schedule_type TEXT NOT NULL, -- 'daily', 'weekly', 'monthly'
    schedule_config TEXT, -- JSON config for schedule
    is_active BOOLEAN DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT
);

-- Recurring task runs table
CREATE TABLE IF NOT EXISTS recurring_task_runs (
    id TEXT PRIMARY KEY,
    recurring_task_id TEXT NOT NULL,
    task_id TEXT,
    scheduled_at TEXT NOT NULL,
    executed_at TEXT,
    status TEXT DEFAULT 'PENDING', -- 'PENDING', 'EXECUTED', 'FAILED'
    error_message TEXT,
    FOREIGN KEY (recurring_task_id) REFERENCES recurring_tasks(id),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- Chat messages table
CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    agent_id TEXT,
    message TEXT NOT NULL,
    message_type TEXT DEFAULT 'USER', -- 'USER', 'AGENT', 'SYSTEM'
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT
);

-- Models table
CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider TEXT NOT NULL,
    model_type TEXT NOT NULL, -- 'text', 'image', 'audio', etc.
    capabilities TEXT, -- JSON array of capabilities
    pricing_info TEXT, -- JSON pricing information
    is_available BOOLEAN DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Feedback table
CREATE TABLE IF NOT EXISTS feedback (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    feedback_type TEXT NOT NULL, -- 'USER_RATING', 'PERFORMANCE', 'ERROR', etc.
    rating REAL,
    comment TEXT,
    context TEXT,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Agent parameter history table
CREATE TABLE IF NOT EXISTS agent_parameter_history (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    parameter_name TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    changed_by TEXT,
    change_reason TEXT,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Collaboration teams table
CREATE TABLE IF NOT EXISTS collaboration_teams (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1,
    created_by TEXT
);

-- Team members table
CREATE TABLE IF NOT EXISTS team_members (
    id TEXT PRIMARY KEY,
    team_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    role TEXT DEFAULT 'MEMBER', -- 'LEADER', 'MEMBER', 'OBSERVER'
    joined_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (team_id) REFERENCES collaboration_teams(id),
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Team tasks table
CREATE TABLE IF NOT EXISTS team_tasks (
    id TEXT PRIMARY KEY,
    team_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    assigned_at TEXT DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'ASSIGNED',
    FOREIGN KEY (team_id) REFERENCES collaboration_teams(id),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);
CREATE INDEX IF NOT EXISTS idx_agents_created_by ON agents(created_by);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_assignee_id ON tasks(assignee_id);
CREATE INDEX IF NOT EXISTS idx_tasks_created_by ON tasks(created_by);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_security_events_timestamp ON security_events(created_at);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_timestamp ON performance_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_entity ON performance_metrics(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_comments_task_id ON comments(task_id);
CREATE INDEX IF NOT EXISTS idx_activity_timestamp ON activity(timestamp);
CREATE INDEX IF NOT EXISTS idx_task_activity_task_id ON task_activity(task_id);
CREATE INDEX IF NOT EXISTS idx_deliverables_task_id ON deliverables(task_id);
CREATE INDEX IF NOT EXISTS idx_recurring_task_runs_recurring_task_id ON recurring_task_runs(recurring_task_id);
CREATE INDEX IF NOT EXISTS idx_chat_messages_created_at ON chat_messages(created_at);
CREATE INDEX IF NOT EXISTS idx_feedback_agent_id ON feedback(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_parameter_history_agent_id ON agent_parameter_history(agent_id);
CREATE INDEX IF NOT EXISTS idx_team_members_team_id ON team_members(team_id);
CREATE INDEX IF NOT EXISTS idx_team_members_agent_id ON team_members(agent_id);
CREATE INDEX IF NOT EXISTS idx_team_tasks_team_id ON team_tasks(team_id);

-- Insert default data
INSERT OR IGNORE INTO models (id, name, provider, model_type, capabilities, is_available) VALUES
('claude-3-sonnet', 'Claude 3 Sonnet', 'Anthropic', 'text', '["text-generation", "analysis", "writing"]', 1),
('claude-3-haiku', 'Claude 3 Haiku', 'Anthropic', 'text', '["text-generation", "analysis", "writing"]', 1),
('gpt-4', 'GPT-4', 'OpenAI', 'text', '["text-generation", "analysis", "writing"]', 1),
('gpt-3.5-turbo', 'GPT-3.5 Turbo', 'OpenAI', 'text', '["text-generation", "analysis", "writing"]', 1);

-- Create triggers for automatic timestamp updates
CREATE TRIGGER IF NOT EXISTS update_agents_timestamp 
    AFTER UPDATE ON agents
    BEGIN
        UPDATE agents SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_tasks_timestamp 
    AFTER UPDATE ON tasks
    BEGIN
        UPDATE tasks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_users_timestamp 
    AFTER UPDATE ON users
    BEGIN
        UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_sessions_last_accessed 
    AFTER UPDATE ON sessions
    BEGIN
        UPDATE sessions SET last_accessed = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_comments_timestamp 
    AFTER UPDATE ON comments
    BEGIN
        UPDATE comments SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_announcements_timestamp 
    AFTER UPDATE ON announcements
    BEGIN
        UPDATE announcements SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_deliverables_timestamp 
    AFTER UPDATE ON deliverables
    BEGIN
        UPDATE deliverables SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_recurring_tasks_timestamp 
    AFTER UPDATE ON recurring_tasks
    BEGIN
        UPDATE recurring_tasks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_models_timestamp 
    AFTER UPDATE ON models
    BEGIN
        UPDATE models SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_collaboration_teams_timestamp 
    AFTER UPDATE ON collaboration_teams
    BEGIN
        UPDATE collaboration_teams SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

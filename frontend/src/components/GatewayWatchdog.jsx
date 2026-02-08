import { useEffect, useState } from 'react'
import { api } from '../api'
import './GatewayWatchdog.css'

export default function GatewayWatchdog() {
  const [isOpen, setIsOpen] = useState(false)
  const [watchdogStatus, setWatchdogStatus] = useState(null)
  const [loading, setLoading] = useState(false)
  const [healthCheckLoading, setHealthCheckLoading] = useState(false)
  const [restartLoading, setRestartLoading] = useState(false)
  const [lastHealthCheck, setLastHealthCheck] = useState(null)

  // Auto-refresh watchdog status every 30 seconds
  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const status = await api.get('/api/monitoring/gateway/status')
        setWatchdogStatus(status)
      } catch (error) {
        console.error('Failed to fetch gateway watchdog status:', error)
      }
    }
    
    fetchStatus()
    const interval = setInterval(fetchStatus, 30 * 1000) // 30 seconds
    
    return () => clearInterval(interval)
  }, [])

  const runHealthCheck = async () => {
    setHealthCheckLoading(true)
    try {
      const result = await api.post('/api/monitoring/gateway/health-check')
      setLastHealthCheck(result)
      
      // Refresh status after health check
      const status = await api.get('/api/monitoring/gateway/status')
      setWatchdogStatus(status)
      
      // Show panel if gateway is down
      if (!result.is_healthy) {
        setIsOpen(true)
      }
    } catch (error) {
      console.error('Failed to run health check:', error)
    } finally {
      setHealthCheckLoading(false)
    }
  }

  const restartGateway = async () => {
    setRestartLoading(true)
    try {
      const result = await api.post('/api/monitoring/gateway/restart')
      
      // Refresh status after restart attempt
      setTimeout(async () => {
        const status = await api.get('/api/monitoring/gateway/status')
        setWatchdogStatus(status)
      }, 2000)
      
      // Show result
      if (result.success) {
        alert('✅ Gateway restart successful!')
      } else {
        alert('❌ Gateway restart failed: ' + result.message)
      }
    } catch (error) {
      console.error('Failed to restart gateway:', error)
      alert('❌ Restart request failed: ' + error.message)
    } finally {
      setRestartLoading(false)
    }
  }

  const formatTimeAgo = (timestamp) => {
    if (!timestamp) return 'Never'
    const ago = new Date() - new Date(timestamp)
    const hours = Math.floor(ago / (1000 * 60 * 60))
    const minutes = Math.floor((ago % (1000 * 60 * 60)) / (1000 * 60))
    
    if (hours > 0) {
      return `${hours}h ${minutes}m ago`
    }
    return `${minutes}m ago`
  }

  const formatDuration = (hours) => {
    if (!hours) return '0m'
    if (hours < 1) {
      return `${Math.round(hours * 60)}m`
    }
    const h = Math.floor(hours)
    const m = Math.round((hours - h) * 60)
    return m > 0 ? `${h}h ${m}m` : `${h}h`
  }

  const getHealthStatusIcon = (status) => {
    switch (status) {
      case 'healthy': return '✅'
      case 'crashed': return '🔴'
      case 'down': return '🔴'
      case 'unknown': return '❓'
      default: return '⚠️'
    }
  }

  const getHealthStatusColor = (status) => {
    switch (status) {
      case 'healthy': return '#22c55e'
      case 'crashed': return '#ef4444'
      case 'down': return '#ef4444'
      case 'unknown': return '#94a3b8'
      default: return '#f59e0b'
    }
  }

  const isGatewayHealthy = watchdogStatus?.health_status === 'healthy'
  const hasIssues = !isGatewayHealthy && watchdogStatus?.health_status !== 'unknown'

  return (
    <>
      {/* Gateway Status Widget */}
      <div className={`gateway-widget ${hasIssues ? 'has-issues' : ''}`}>
        <button 
          className="widget-toggle"
          onClick={() => setIsOpen(!isOpen)}
          title="Gateway Watchdog"
        >
          <span className="gateway-icon">
            {hasIssues ? '🔴' : '🟢'}
          </span>
          <span className="gateway-label">Gateway</span>
          {hasIssues && (
            <span className="issue-badge">!</span>
          )}
        </button>
      </div>

      {/* Gateway Panel */}
      {isOpen && (
        <div className="gateway-modal">
          <div className="gateway-content">
            <div className="gateway-header">
              <h2>🛡️ Gateway Watchdog</h2>
              <button className="close-btn" onClick={() => setIsOpen(false)}>×</button>
            </div>

            <div className="gateway-controls">
              <button 
                className="health-check-btn"
                onClick={runHealthCheck}
                disabled={healthCheckLoading}
              >
                {healthCheckLoading ? '⏳ Checking...' : '🔍 Health Check'}
              </button>
              <button 
                className="restart-btn"
                onClick={restartGateway}
                disabled={restartLoading || isGatewayHealthy}
                title={isGatewayHealthy ? 'Gateway is healthy' : 'Restart gateway'}
              >
                {restartLoading ? '⏳ Restarting...' : '🔄 Restart'}
              </button>
            </div>

            {/* Current Health Status */}
            {watchdogStatus && (
              <div className="health-status-section">
                <h3>Current Status</h3>
                <div className="health-status-card">
                  <div className="health-indicator">
                    <span 
                      className="status-icon"
                      style={{ color: getHealthStatusColor(watchdogStatus.health_status) }}
                    >
                      {getHealthStatusIcon(watchdogStatus.health_status)}
                    </span>
                    <span 
                      className="status-text"
                      style={{ color: getHealthStatusColor(watchdogStatus.health_status) }}
                    >
                      {watchdogStatus.health_status?.toUpperCase() || 'UNKNOWN'}
                    </span>
                  </div>
                  
                  <div className="health-details">
                    <div className="health-item">
                      <span className="health-label">Monitoring:</span>
                      <span className="health-value">
                        {watchdogStatus.monitoring ? '🟢 Active' : '🔴 Inactive'}
                      </span>
                    </div>
                    <div className="health-item">
                      <span className="health-label">Last Check:</span>
                      <span className="health-value">
                        {formatTimeAgo(watchdogStatus.last_check)}
                      </span>
                    </div>
                    {watchdogStatus.time_since_healthy_minutes !== null && (
                      <div className="health-item">
                        <span className="health-label">Time Since Healthy:</span>
                        <span className={`health-value ${watchdogStatus.time_since_healthy_minutes > 5 ? 'warning' : ''}`}>
                          {watchdogStatus.time_since_healthy_minutes < 1 ? 'Just now' : 
                           `${Math.round(watchdogStatus.time_since_healthy_minutes)}m ago`}
                        </span>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            )}

            {/* Uptime Statistics */}
            {watchdogStatus && (
              <div className="uptime-section">
                <h3>📊 Uptime Statistics</h3>
                <div className="uptime-stats">
                  <div className="stat-item">
                    <span className="stat-label">Current Session:</span>
                    <span className="stat-value">
                      {formatDuration(watchdogStatus.current_uptime_hours)}
                    </span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-label">Total Uptime:</span>
                    <span className="stat-value">
                      {formatDuration(watchdogStatus.total_uptime_hours)}
                    </span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-label">Crash Count:</span>
                    <span className={`stat-value ${watchdogStatus.crash_count > 0 ? 'warning' : ''}`}>
                      {watchdogStatus.crash_count}
                    </span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-label">Auto Restarts:</span>
                    <span className="stat-value">
                      {watchdogStatus.restart_count}
                    </span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-label">Consecutive Failures:</span>
                    <span className={`stat-value ${watchdogStatus.consecutive_failures > 0 ? 'error' : ''}`}>
                      {watchdogStatus.consecutive_failures}
                    </span>
                  </div>
                </div>
              </div>
            )}

            {/* Recent Events */}
            {watchdogStatus && (watchdogStatus.last_crash || watchdogStatus.last_notification) && (
              <div className="events-section">
                <h3>🕐 Recent Events</h3>
                <div className="events-list">
                  {watchdogStatus.last_crash && (
                    <div className="event-item crash">
                      <span className="event-icon">💥</span>
                      <div className="event-content">
                        <div className="event-title">Last Crash</div>
                        <div className="event-time">{formatTimeAgo(watchdogStatus.last_crash)}</div>
                      </div>
                    </div>
                  )}
                  {watchdogStatus.last_notification && (
                    <div className="event-item notification">
                      <span className="event-icon">📢</span>
                      <div className="event-content">
                        <div className="event-title">Last Notification</div>
                        <div className="event-time">{formatTimeAgo(watchdogStatus.last_notification)}</div>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Last Health Check Result */}
            {lastHealthCheck && (
              <div className="health-check-section">
                <h3>🔍 Last Health Check</h3>
                <div className={`health-check-result ${lastHealthCheck.is_healthy ? 'healthy' : 'unhealthy'}`}>
                  <div className="health-check-status">
                    <span className="health-check-icon">
                      {lastHealthCheck.is_healthy ? '✅' : '❌'}
                    </span>
                    <span className="health-check-text">
                      {lastHealthCheck.status_message}
                    </span>
                  </div>
                  <div className="health-check-time">
                    {formatTimeAgo(lastHealthCheck.check_time)}
                  </div>
                </div>
              </div>
            )}

            {/* Configuration */}
            {watchdogStatus?.config && (
              <div className="config-section">
                <h3>⚙️ Configuration</h3>
                <div className="config-grid">
                  <div className="config-item">
                    <span className="config-label">Check Interval:</span>
                    <span className="config-value">{watchdogStatus.config.check_interval_seconds}s</span>
                  </div>
                  <div className="config-item">
                    <span className="config-label">Health Timeout:</span>
                    <span className="config-value">{watchdogStatus.config.health_check_timeout}s</span>
                  </div>
                  <div className="config-item">
                    <span className="config-label">Max Restart Attempts:</span>
                    <span className="config-value">{watchdogStatus.config.max_restart_attempts}</span>
                  </div>
                  <div className="config-item">
                    <span className="config-label">Notification Cooldown:</span>
                    <span className="config-value">{watchdogStatus.config.notification_cooldown_minutes}m</span>
                  </div>
                </div>
              </div>
            )}

            {/* Healthy Status */}
            {isGatewayHealthy && (
              <div className="healthy-status">
                <div className="success-icon">✅</div>
                <h3>Gateway Healthy!</h3>
                <p>OpenClaw gateway is running normally.</p>
              </div>
            )}
          </div>
        </div>
      )}
    </>
  )
}
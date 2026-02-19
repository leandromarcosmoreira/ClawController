import { Users, Bot, Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useMissionStore } from '../store/useMissionStore'

const roleClasses = {
  LEAD: 'badge badge-lead',
  INT: 'badge badge-int',
  SPC: 'badge badge-spc'
}

const statusConfig = {
  WORKING: { dot: 'status-dot--green', label: 'sidebar.status_working', pulse: true },
  IDLE: { dot: 'status-dot--yellow', label: 'sidebar.status_idle', pulse: false },
  STANDBY: { dot: 'status-dot--blue', label: 'sidebar.status_standby', pulse: false },
  OFFLINE: { dot: 'status-dot--gray', label: 'sidebar.status_offline', pulse: false },
  ERROR: { dot: 'status-dot--red', label: 'sidebar.status_error', pulse: true }
}

export default function AgentSidebar() {
  const { t } = useTranslation()
  const agents = useMissionStore((state) => state.agents)
  const selectedAgentId = useMissionStore((state) => state.selectedAgentId)
  const toggleAgentFilter = useMissionStore((state) => state.toggleAgentFilter)
  const openAgentManagement = useMissionStore((state) => state.openAgentManagement)
  const activeAgents = agents.filter((agent) => agent.status === 'WORKING').length

  // Show empty state when no agents exist
  if (agents.length === 0) {
    return (
      <aside className="sidebar">
        <div className="sidebar-header">
          <div className="sidebar-title">
            <Users size={16} />
            {t('sidebar.title')}
          </div>
          <span className="count-badge">0</span>
        </div>

        <div className="empty-state">
          <div className="empty-state-icon">
            <Bot size={48} />
          </div>
          <h3 className="empty-state-title">{t('sidebar.welcome_title')}</h3>
          <p className="empty-state-description">
            {t('sidebar.welcome_description')}
          </p>
          <button
            className="empty-state-button"
            onClick={openAgentManagement}
          >
            <Plus size={16} />
            {t('sidebar.create_first_agent')}
          </button>
          <div className="empty-state-tips">
            <h4>{t('sidebar.quick_tips')}</h4>
            <ul>
              <li>{t('sidebar.tip1')}</li>
              <li>{t('sidebar.tip2')}</li>
              <li>{t('sidebar.tip3')}</li>
            </ul>
          </div>
        </div>
      </aside>
    )
  }

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <div className="sidebar-title">
          <Users size={16} />
          {t('sidebar.title')}
        </div>
        <span className="count-badge">{agents.length}</span>
      </div>

      <div className="sidebar-summary">
        <div>
          <div className="summary-title">{t('sidebar.all_agents')}</div>
          <div className="summary-subtitle">
            {selectedAgentId ? t('sidebar.click_to_clear') : t('sidebar.click_to_filter')}
          </div>
        </div>
        <div className="summary-count">{activeAgents}</div>
      </div>

      <div className="agent-list">
        {agents.map((agent) => {
          const isSelected = selectedAgentId === agent.id
          return (
            <button
              key={agent.id}
              type="button"
              className={`agent-card ${isSelected ? 'agent-card--selected' : ''}`}
              onClick={() => toggleAgentFilter(agent.id)}
              style={isSelected ? {
                borderColor: agent.color,
                boxShadow: `0 0 0 2px ${agent.color}25, 0 10px 20px rgba(224, 123, 60, 0.12)`
              } : undefined}
            >
              <div className="agent-avatar" style={{ backgroundColor: agent.color }}>
                <span>{agent.avatar}</span>
              </div>
              <div className="agent-info">
                <div className="agent-top">
                  <span className="agent-name">{agent.name}</span>
                  {agent.role === 'LEAD' && <span className={roleClasses[agent.role]}>{t('sidebar.role_lead')}</span>}
                </div>
                <div className="agent-desc">{agent.description}</div>
              </div>
              <div className="agent-status" title={t(statusConfig[agent.status]?.label || agent.status)}>
                <span className={`status-dot ${statusConfig[agent.status]?.dot || 'status-dot--gray'} ${statusConfig[agent.status]?.pulse ? 'status-dot--pulse' : ''}`} />
                <span className="status-label">{t(statusConfig[agent.status]?.label || agent.status)}</span>
              </div>
            </button>
          )
        })}
      </div>
    </aside>
  )
}

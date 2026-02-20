import { useState, useEffect } from 'react'
import { Download, CheckSquare, Square, Bot, AlertCircle } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useMissionStore } from '../store/useMissionStore'

// Status indicator colors
const statusConfig = {
  WORKING: { color: '#22C55E', label: 'agent_management.status_working', dotClass: 'status-dot--green status-dot--pulse' },
  IDLE: { color: '#F59E0B', label: 'agent_management.status_idle', dotClass: 'status-dot--yellow' },
  STANDBY: { color: '#9CA3AF', label: 'agent_management.status_standby', dotClass: 'status-dot--gray' },
  OFFLINE: { color: '#EF4444', label: 'agent_management.status_offline', dotClass: 'status-dot--red' },
}

function AgentImportCard({ agent, isSelected, onToggle, isAlreadyExists }) {
  const { t } = useTranslation()
  const status = statusConfig[agent.status] || statusConfig.OFFLINE

  return (
    <div className={`import-agent-card ${isSelected ? 'import-agent-card--selected' : ''} ${isAlreadyExists ? 'import-agent-card--exists' : ''}`}>
      <div className="import-agent-checkbox">
        <button
          className="checkbox-button"
          onClick={() => !isAlreadyExists && onToggle(agent.id)}
          disabled={isAlreadyExists}
        >
          {isSelected && !isAlreadyExists ? <CheckSquare size={20} /> : <Square size={20} />}
        </button>
      </div>

      <div className="import-agent-info">
        <div className="import-agent-header">
          <div className="import-agent-avatar" style={{ background: agent.color || 'var(--accent)' }}>
            {agent.avatar || agent.emoji || '🤖'}
          </div>
          <div className="import-agent-details">
            <h4>{agent.name}</h4>
            <span className="import-agent-id">@{agent.id}</span>
          </div>
          <div className="import-agent-status">
            <span className={`status-dot ${status.dotClass}`} />
            <span className="status-label">{t(status.label)}</span>
          </div>
        </div>

        {agent.description && (
          <p className="import-agent-description">{agent.description}</p>
        )}

        {agent.workspace && (
          <div className="import-agent-workspace">
            <span className="workspace-label">{t('agent_management.workspace')}:</span>
            <span className="workspace-path">{agent.workspace}</span>
          </div>
        )}
      </div>

      {isAlreadyExists && (
        <div className="import-agent-exists">
          <AlertCircle size={16} />
          {t('agent_management.already_exists')}
        </div>
      )}
    </div>
  )
}

export default function ImportAgentsDialog() {
  const { t } = useTranslation()
  const isOpen = useMissionStore((s) => s.isImportDialogOpen)
  const closeImportDialog = useMissionStore((s) => s.closeImportDialog)
  const fetchOpenClawAgents = useMissionStore((s) => s.fetchOpenClawAgents)
  const importAgentsFromOpenClaw = useMissionStore((s) => s.importAgentsFromOpenClaw)
  const openClawAgents = useMissionStore((s) => s.openClawAgents)
  const existingAgents = useMissionStore((s) => s.agents)


  const [selectedAgents, setSelectedAgents] = useState(new Set())
  const [loading, setLoading] = useState(false)
  const [importing, setImporting] = useState(false)
  const [importResult, setImportResult] = useState(null)

  // Load OpenClaw agents when dialog opens
  useEffect(() => {
    if (isOpen) {
      setLoading(true)
      fetchOpenClawAgents().finally(() => setLoading(false))
      setSelectedAgents(new Set())
      setImportResult(null)
    }
  }, [isOpen, fetchOpenClawAgents])

  // Filter agents to show import candidates
  const existingAgentIds = new Set(existingAgents.map(a => a.id))
  const importCandidates = openClawAgents.filter(agent => !existingAgentIds.has(agent.id))
  const alreadyExists = openClawAgents.filter(agent => existingAgentIds.has(agent.id))

  const handleToggleAgent = (agentId) => {
    const newSelection = new Set(selectedAgents)
    if (newSelection.has(agentId)) {
      newSelection.delete(agentId)
    } else {
      newSelection.add(agentId)
    }
    setSelectedAgents(newSelection)
  }

  const handleSelectAll = () => {
    if (selectedAgents.size === importCandidates.length) {
      // Deselect all
      setSelectedAgents(new Set())
    } else {
      // Select all candidates
      setSelectedAgents(new Set(importCandidates.map(a => a.id)))
    }
  }

  const handleImport = async () => {
    if (selectedAgents.size === 0) return

    setImporting(true)
    try {
      const result = await importAgentsFromOpenClaw(Array.from(selectedAgents))
      setImportResult(result)
      setSelectedAgents(new Set()) // Clear selection after import
    } catch (error) {
      console.error('Import failed:', error)
      alert(t('agent_management.import_dialog.import_failed', { message: error.message }))
    } finally {
      setImporting(false)
    }
  }

  if (!isOpen) return null

  return (
    <>
      <div className="import-dialog-overlay" onClick={closeImportDialog} />
      <div className="import-dialog">
        <div className="import-dialog-header">
          <div className="import-dialog-title">
            <Download size={24} />
            <h2>{t('agent_management.import_dialog.title')}</h2>
          </div>
          <button className="import-dialog-close" onClick={closeImportDialog}>
            ×
          </button>
        </div>

        <div className="import-dialog-body">
          {loading && (
            <div className="import-loading">
              <Bot size={32} />
              <p>{t('agent_management.import_dialog.loading')}</p>
            </div>
          )}

          {!loading && openClawAgents.length === 0 && (
            <div className="import-empty">
              <AlertCircle size={32} />
              <h3>{t('agent_management.import_dialog.empty_title')}</h3>
              <p>{t('agent_management.import_dialog.empty_hint')}</p>
            </div>
          )}

          {!loading && openClawAgents.length > 0 && !importResult && (
            <>
              <div className="import-summary">
                <p dangerouslySetInnerHTML={{
                  __html: t('agent_management.import_dialog.summary', {
                    total: openClawAgents.length,
                    candidates: importCandidates.length,
                    existing: alreadyExists.length
                  })
                }} />

                {importCandidates.length > 0 && (
                  <div className="import-controls">
                    <button
                      className="select-all-button"
                      onClick={handleSelectAll}
                      disabled={importing}
                    >
                      {selectedAgents.size === importCandidates.length
                        ? t('agent_management.import_dialog.deselect_all')
                        : t('agent_management.import_dialog.select_all')}
                    </button>
                    <span className="selection-count">
                      {t('agent_management.import_dialog.selection_count', {
                        selected: selectedAgents.size,
                        total: importCandidates.length
                      })}
                    </span>
                  </div>
                )}
              </div>

              <div className="import-agents-list">
                {importCandidates.map(agent => (
                  <AgentImportCard
                    key={agent.id}
                    agent={agent}
                    isSelected={selectedAgents.has(agent.id)}
                    onToggle={handleToggleAgent}
                    isAlreadyExists={false}
                  />
                ))}

                {alreadyExists.map(agent => (
                  <AgentImportCard
                    key={agent.id}
                    agent={agent}
                    isSelected={false}
                    onToggle={() => { }}
                    isAlreadyExists={true}
                  />
                ))}
              </div>
            </>
          )}

          {importResult && (
            <div className="import-result">
              <div className="import-result-header">
                <CheckSquare size={24} className="success-icon" />
                <h3>{t('agent_management.import_dialog.result_success')}</h3>
              </div>

              <div className="import-result-summary">
                <div className="result-stat">
                  <span className="result-number">{importResult.imported_count}</span>
                  <span className="result-label">{t('agent_management.import_dialog.imported')}</span>
                </div>
                <div className="result-stat">
                  <span className="result-number">{importResult.skipped_count}</span>
                  <span className="result-label">{t('agent_management.import_dialog.skipped')}</span>
                </div>
              </div>

              {importResult.imported.length > 0 && (
                <div className="imported-agents">
                  <h4>{t('agent_management.import_dialog.success_list_title')}</h4>
                  <ul>
                    {importResult.imported.map(agent => (
                      <li key={agent.id}>
                        <strong>{agent.name}</strong> (@{agent.id}) - {t(`agent_management.status.${agent.status.toLowerCase()}`, { defaultValue: agent.status })}
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              {importResult.skipped.length > 0 && (
                <div className="skipped-agents">
                  <h4>{t('agent_management.import_dialog.skipped_list_title')}</h4>
                  <ul>
                    {importResult.skipped.map(item => (
                      <li key={item.id}>
                        <strong>{item.id}</strong> - {item.reason}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>

        <div className="import-dialog-footer">
          {!loading && !importResult && importCandidates.length > 0 && (
            <button
              className="import-button"
              onClick={handleImport}
              disabled={selectedAgents.size === 0 || importing}
            >
              {importing ? (
                <>
                  <Bot size={16} className="spin" />
                  {t('agent_management.import_dialog.importing')}
                </>
              ) : (
                <>
                  <Download size={16} />
                  {selectedAgents.size === 1
                    ? t('agent_management.import_dialog.import_button_one', { count: selectedAgents.size })
                    : t('agent_management.import_dialog.import_button', { count: selectedAgents.size })}
                </>
              )}
            </button>
          )}

          {importResult && (
            <button className="close-button" onClick={closeImportDialog}>
              {t('agent_management.import_dialog.close')}
            </button>
          )}

          <button className="cancel-button" onClick={closeImportDialog}>
            {importResult ? t('agent_management.import_dialog.close') : t('agent_management.import_dialog.cancel')}
          </button>
        </div>
      </div>
    </>
  )
}
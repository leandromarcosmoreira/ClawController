import { useState, useEffect } from 'react'
import {
  X, RefreshCw, Play, Pause, Trash2, ChevronDown, ChevronRight,
  Clock, Calendar, CheckCircle2, XCircle, RotateCcw
} from 'lucide-react'
import { useMissionStore } from '../store/useMissionStore'
import { fetchRecurringTaskRuns } from '../api'
import { useTranslation } from 'react-i18next'

function RunHistoryItem({ run, agents }) {
  const getStatusIcon = () => {
    if (run.status === 'success') {
      return <CheckCircle2 size={14} className="run-status-icon success" />
    }
    return <XCircle size={14} className="run-status-icon failed" />
  }

  const getTaskStatusColor = () => {
    if (!run.task) return '#6B7280'
    const statusColors = {
      'INBOX': '#E07B3C',
      'ASSIGNED': '#8B5CF6',
      'IN_PROGRESS': '#F97316',
      'REVIEW': '#0EA5E9',
      'DONE': '#22C55E'
    }
    return statusColors[run.task.status] || '#6B7280'
  }

  const formatRunTime = (isoString) => {
    const date = new Date(isoString)
    return date.toLocaleString('pt-BR', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  return (
    <div className="run-history-item">
      {getStatusIcon()}
      <span className="run-time">{formatRunTime(run.run_at)}</span>
      {run.task && (
        <span
          className="run-task-status"
          style={{ color: getTaskStatusColor() }}
        >
          {run.task.status.replace('_', ' ')}
        </span>
      )}
    </div>
  )
}

function RecurringTaskItem({ task, isSelected, onSelect, agents }) {
  const { t } = useTranslation()
  const toggleRecurringTask = useMissionStore((state) => state.toggleRecurringTask)
  const deleteRecurringTask = useMissionStore((state) => state.deleteRecurringTask)
  const triggerRecurringTask = useMissionStore((state) => state.triggerRecurringTask)
  const selectTask = useMissionStore((state) => state.selectTask)

  const [runs, setRuns] = useState([])
  const [loadingRuns, setLoadingRuns] = useState(false)
  const [triggering, setTriggering] = useState(false)

  useEffect(() => {
    if (isSelected) {
      setLoadingRuns(true)
      fetchRecurringTaskRuns(task.id, 10)
        .then(setRuns)
        .catch(console.error)
        .finally(() => setLoadingRuns(false))
    }
  }, [isSelected, task.id, task.run_count])

  const handleToggle = (e) => {
    e.stopPropagation()
    toggleRecurringTask(task.id)
  }

  const handleDelete = (e) => {
    e.stopPropagation()
    if (confirm(`Excluir tarefa recorrente "${task.title}"?`)) {
      deleteRecurringTask(task.id)
    }
  }

  const handleTrigger = async (e) => {
    e.stopPropagation()
    setTriggering(true)
    try {
      await triggerRecurringTask(task.id)
    } catch (error) {
      alert('Falha ao executar tarefa')
    }
    setTriggering(false)
  }

  const formatNextRun = () => {
    if (!task.next_run_at) return 'Não agendado'
    const date = new Date(task.next_run_at)
    const now = new Date()
    const diffMs = date - now
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
    const diffMins = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60))

    if (diffMs < 0) return t('recurring.overdue')
    if (diffHours === 0) return t('recurring.in_minutes', { m: diffMins })
    if (diffHours < 24) return t('recurring.in_hours_minutes', { h: diffHours, m: diffMins })
    const diffDays = Math.floor(diffHours / 24)
    return t('recurring.in_days_hours', { d: diffDays, h: diffHours % 24 })
  }

  const formatLastRun = () => {
    if (!task.last_run_at) return 'Nunca'
    const date = new Date(task.last_run_at)
    return date.toLocaleDateString('pt-BR', { month: 'short', day: 'numeric' })
  }

  const assignee = agents.find(a => a.id === task.assignee_id)

  return (
    <div className={`recurring-task-item ${isSelected ? 'expanded' : ''} ${!task.is_active ? 'paused' : ''}`}>
      <div className="recurring-task-header" onClick={() => onSelect(isSelected ? null : task.id)}>
        <div className="recurring-task-expand">
          {isSelected ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        </div>

        <div className="recurring-task-info">
          <div className="recurring-task-title">
            {task.title}
            {task.priority === 'URGENT' && <span className="priority-badge urgent">🔴</span>}
          </div>
          <div className="recurring-task-schedule">
            <Calendar size={12} />
            {task.schedule_human}
          </div>
        </div>

        <div className="recurring-task-meta">
          <div className={`recurring-task-status ${task.is_active ? 'active' : 'paused'}`}>
            {task.is_active ? 'Ativo' : 'Pausado'}
          </div>
          <div className="recurring-task-next">
            <Clock size={12} />
            {formatNextRun()}
          </div>
        </div>

        <div className="recurring-task-actions">
          <button
            className="recurring-action-btn trigger"
            onClick={handleTrigger}
            disabled={triggering}
            title="Executar Agora"
          >
            {triggering ? <RotateCcw size={14} className="spin" /> : <Play size={14} />}
          </button>
          <button
            className="recurring-action-btn toggle"
            onClick={handleToggle}
            title={task.is_active ? 'Pausar' : 'Retomar'}
          >
            {task.is_active ? <Pause size={14} /> : <RefreshCw size={14} />}
          </button>
          <button
            className="recurring-action-btn delete"
            onClick={handleDelete}
            title="Excluir"
          >
            <Trash2 size={14} />
          </button>
        </div>
      </div>

      {isSelected && (
        <div className="recurring-task-details">
          {task.description && (
            <div className="recurring-task-description">{task.description}</div>
          )}

          <div className="recurring-task-stats">
            <div className="stat">
              <span className="stat-label">{t('recurring.run_count')}</span>
              <span className="stat-value">{task.run_count}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('recurring.last_run')}</span>
              <span className="stat-value">{formatLastRun()}</span>
            </div>
            {assignee && (
              <div className="stat">
                <span className="stat-label">{t('recurring.assignee')}</span>
                <span className="stat-value assignee">
                  <span className="assignee-avatar" style={{ backgroundColor: assignee.color }}>
                    {assignee.avatar}
                  </span>
                  {assignee.name}
                </span>
              </div>
            )}
          </div>

          <div className="recurring-task-history">
            <div className="history-header">{t('recurring.run_history')}</div>
            {loadingRuns ? (
              <div className="history-loading">{t('recurring.loading')}</div>
            ) : runs.length === 0 ? (
              <div className="history-empty">{t('recurring.no_runs_yet')}</div>
            ) : (
              <div className="history-list">
                {runs.map(run => (
                  <RunHistoryItem key={run.id} run={run} agents={agents} />
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}

export default function RecurringTasksPanel() {
  const isOpen = useMissionStore((state) => state.isRecurringPanelOpen)
  const closePanel = useMissionStore((state) => state.closeRecurringPanel)
  const recurringTasks = useMissionStore((state) => state.recurringTasks)
  const agents = useMissionStore((state) => state.agents)
  const selectedRecurringTaskId = useMissionStore((state) => state.selectedRecurringTaskId)
  const selectRecurringTask = useMissionStore((state) => state.selectRecurringTask)
  const refreshRecurringTasks = useMissionStore((state) => state.refreshRecurringTasks)

  if (!isOpen) return null

  const activeCount = recurringTasks.filter(t => t.is_active).length

  return (
    <div className="recurring-panel-overlay" onClick={(e) => e.target === e.currentTarget && closePanel()}>
      <div className="recurring-panel">
        <div className="recurring-panel-header">
          <div className="recurring-panel-title">
            <RefreshCw size={20} />
            <span>Tarefas Recorrentes</span>
            <span className="recurring-count-badge">{activeCount} ativas</span>
          </div>
          <div className="recurring-panel-actions">
            <button className="icon-button" onClick={refreshRecurringTasks} title="Atualizar">
              <RotateCcw size={16} />
            </button>
            <button className="icon-button" onClick={closePanel}>
              <X size={18} />
            </button>
          </div>
        </div>

        <div className="recurring-panel-content">
          {recurringTasks.length === 0 ? (
            <div className="recurring-empty">
              <RefreshCw size={48} />
              <p>Nenhuma tarefa recorrente ainda</p>
              <span>Crie uma tarefa e ative "Tornar Recorrente" para configurar tarefas automatizadas</span>
            </div>
          ) : (
            <div className="recurring-list">
              {recurringTasks.map(task => (
                <RecurringTaskItem
                  key={task.id}
                  task={task}
                  isSelected={selectedRecurringTaskId === task.id}
                  onSelect={selectRecurringTask}
                  agents={agents}
                />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

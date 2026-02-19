import { Plus, Bot, Rocket, Zap, Target } from 'lucide-react'
import { DndContext, DragOverlay, closestCenter, PointerSensor, useSensor, useSensors } from '@dnd-kit/core'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { statusColors, statusOrder, useMissionStore } from '../store/useMissionStore'
import KanbanColumn from './KanbanColumn'
import TaskCard from './TaskCard'

export default function KanbanBoard() {
  const { t } = useTranslation()
  const tasks = useMissionStore((state) => state.tasks)
  const selectedAgentId = useMissionStore((state) => state.selectedAgentId)
  const moveTask = useMissionStore((state) => state.moveTask)
  const openNewTask = useMissionStore((state) => state.openNewTask)
  const openAgentManagement = useMissionStore((state) => state.openAgentManagement)
  const agents = useMissionStore((state) => state.agents)

  const [activeTask, setActiveTask] = useState(null)

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    })
  )

  // Filter tasks by selected agent
  const filteredTasks = selectedAgentId
    ? tasks.filter(task => task.assignedTo === selectedAgentId)
    : tasks

  const selectedAgent = selectedAgentId
    ? agents.find(a => a.id === selectedAgentId)
    : null

  const handleDragStart = (event) => {
    const { active } = event
    const task = tasks.find(t => t.id === active.id)
    setActiveTask(task)
  }

  const handleDragEnd = (event) => {
    const { active, over } = event
    setActiveTask(null)

    if (!over) return

    const taskId = active.id
    const newStatus = over.id

    // Validate that we're dropping on a valid status column
    if (statusOrder.includes(newStatus)) {
      moveTask(taskId, newStatus)
    }
  }

  // Show empty state when no agents exist
  if (agents.length === 0) {
    return (
      <section className="kanban">
        <div className="kanban-toolbar">
          <button className="add-task-button" onClick={openAgentManagement}>
            <Bot size={16} />
            {t('sidebar.create_first_agent')}
          </button>
        </div>

        <div className="kanban-empty-state">
          <div className="empty-dashboard">
            <div className="empty-dashboard-icon">
              <Rocket size={64} />
            </div>
            <h2 className="empty-dashboard-title">{t('kanban.empty.title')}</h2>
            <p className="empty-dashboard-description">
              {t('kanban.empty.description')}
            </p>

            <div className="getting-started-steps">
              <div className="step">
                <div className="step-icon">
                  <Bot size={24} />
                </div>
                <div className="step-content">
                  <h3>{t('kanban.empty.step1_title')}</h3>
                  <p>{t('kanban.empty.step1_description')}</p>
                </div>
              </div>

              <div className="step">
                <div className="step-icon">
                  <Target size={24} />
                </div>
                <div className="step-content">
                  <h3>{t('kanban.empty.step2_title')}</h3>
                  <p>{t('kanban.empty.step2_description')}</p>
                </div>
              </div>

              <div className="step">
                <div className="step-icon">
                  <Zap size={24} />
                </div>
                <div className="step-content">
                  <h3>{t('kanban.empty.step3_title')}</h3>
                  <p>{t('kanban.empty.step3_description')}</p>
                </div>
              </div>
            </div>

            <button
              className="primary-cta-button"
              onClick={openAgentManagement}
            >
              <Plus size={20} />
              {t('kanban.empty.create_first_agent')}
            </button>
          </div>
        </div>
      </section>
    )
  }

  return (
    <section className="kanban">
      <div className="kanban-toolbar">
        {selectedAgent && (
          <span className="filter-indicator">
            {t('kanban.filtering_by')} <span className="filter-agent" style={{ color: selectedAgent.color }}>
              {selectedAgent.avatar} {selectedAgent.name}
            </span>
            <button className="clear-filter" onClick={() => useMissionStore.getState().clearAgentFilter()}>
              {t('kanban.clear_filter')}
            </button>
          </span>
        )}
        <button className="add-task-button" onClick={openNewTask}>
          <Plus size={16} />
          {t('kanban.new_task')}
        </button>
        <button className="add-task-button" onClick={openAgentManagement} style={{ marginLeft: '8px' }}>
          <Bot size={16} />
          {t('kanban.manage_agents')}
        </button>
      </div>

      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
      >
        <div className="kanban-grid">
          {statusOrder.map((status) => {
            const columnTasks = filteredTasks.filter((task) => task.status === status)
            return (
              <KanbanColumn
                key={status}
                title={status}
                tasks={columnTasks}
                color={statusColors[status]}
              />
            )
          })}
        </div>

        <DragOverlay>
          {activeTask ? (
            <div className="task-card-overlay">
              <TaskCard task={activeTask} isDragging isOverlay />
            </div>
          ) : null}
        </DragOverlay>
      </DndContext>
    </section>
  )
}

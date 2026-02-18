import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import './App.css'
import clawLogo from './assets/clawcontroller-logo.jpg'
import AgentManagement from './components/AgentManagement'
import AgentSidebar from './components/AgentSidebar'
import AnnouncementModal from './components/AnnouncementModal'
import ChatWidget from './components/ChatWidget'
import Header from './components/Header'
import KanbanBoard from './components/KanbanBoard'
import LiveFeed from './components/LiveFeed'
import NewTaskModal from './components/NewTaskModal'
import RecurringTasksPanel from './components/RecurringTasksPanel'
import TaskModal from './components/TaskModal'
import { useMissionStore } from './store/useMissionStore'

function LoadingScreen() {
  const { t } = useTranslation()
  
  return (
    <div className="loading-screen">
      <div className="loading-content">
        <img src={clawLogo} alt="ClawController" className="loading-logo" />
        <h2>{t('app.title')}</h2>
        <p>{t('app.loading')}</p>
      </div>
    </div>
  )
}

function ErrorScreen({ error, onRetry }) {
  const { t } = useTranslation()
  
  return (
    <div className="error-screen">
      <div className="error-content">
        <div className="error-icon">⚠️</div>
        <h2>{t('app.error.title')}</h2>
        <p>{error}</p>
        <button className="retry-button" onClick={onRetry}>
          {t('app.error.retry')}
        </button>
        <p className="error-hint">
          {t('app.error.hint')}
        </p>
      </div>
    </div>
  )
}

function App() {
  const initialize = useMissionStore((state) => state.initialize)
  const connectWebSocket = useMissionStore((state) => state.connectWebSocket)
  const disconnectWebSocket = useMissionStore((state) => state.disconnectWebSocket)
  const refreshAgents = useMissionStore((state) => state.refreshAgents)
  const isLoading = useMissionStore((state) => state.isLoading)
  const isInitialized = useMissionStore((state) => state.isInitialized)
  const error = useMissionStore((state) => state.error)
  const wsConnected = useMissionStore((state) => state.wsConnected)

  useEffect(() => {
    // Initialize data on mount
    initialize()
    
    // Connect WebSocket
    connectWebSocket()
    
    // Refresh agent status every 30 seconds for real-time updates
    const agentRefreshInterval = setInterval(() => {
      refreshAgents()
    }, 30000)
    
    // Cleanup on unmount
    return () => {
      disconnectWebSocket()
      clearInterval(agentRefreshInterval)
    }
  }, [initialize, connectWebSocket, disconnectWebSocket, refreshAgents])

  // Show loading screen while initializing
  if (isLoading && !isInitialized) {
    return <LoadingScreen />
  }

  // Show error screen if initialization failed
  if (error && !isInitialized) {
    return <ErrorScreen error={error} onRetry={initialize} />
  }

  return (
    <div className="app">
      <Header />
      <main className="main">
        <AgentSidebar />
        <KanbanBoard />
        <div className="right-panel">
          <LiveFeed />
        </div>
      </main>
      <TaskModal />
      <AnnouncementModal />
      <NewTaskModal />
      <RecurringTasksPanel />
      <AgentManagement />
      <ChatWidget />
    </div>
  )
}

export default App

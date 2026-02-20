import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter, Routes, Route } from 'react-router-dom'
import './index.css'
import './i18n'
import { Suspense, Component } from 'react'
import App from './App.jsx'
import StatusPage from './components/StatusPage.jsx'

class ErrorBoundary extends Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error) {
    return { hasError: true };
  }

  componentDidCatch(error, errorInfo) {
    this.setState({ error, errorInfo });
    console.error("Uncaught error:", error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{ padding: '20px', color: '#ef4444', backgroundColor: '#18181b', height: '100vh' }}>
          <h1>Something went wrong.</h1>
          <details style={{ whiteSpace: 'pre-wrap', marginTop: '10px' }}>
            {this.state.error && this.state.error.toString()}
            <br />
            {this.state.errorInfo && this.state.errorInfo.componentStack}
          </details>
        </div>
      );
    }

    return this.props.children;
  }
}

// Loading component
const Loading = () => (
  <div style={{
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    height: '100vh',
    backgroundColor: '#09090b',
    color: '#e2e8f0'
  }}>
    <h2>Loading Application...</h2>
  </div>
)

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <Suspense fallback={<Loading />}>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={
            <ErrorBoundary>
              <App />
            </ErrorBoundary>
          } />
          <Route path="/status" element={<StatusPage />} />
        </Routes>
      </BrowserRouter>
    </Suspense>
  </StrictMode>,
)

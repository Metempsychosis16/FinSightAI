import { useState } from 'react'
import LoginPage from './features/auth/pages/LoginPage'
import DashboardPage from './features/dashboard/pages/DashboardPage'
import InvestigationPage from './features/investigation/pages/InvestigationPage'
import { getAccessToken } from './features/auth/services/tokenStorage'

function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(
    () => getAccessToken() !== null,
  )

  const [selectedCaseId, setSelectedCaseId] = useState<number | null>(
    null,
  )

  function handleLogout() {
    setSelectedCaseId(null)
    setIsAuthenticated(false)
  }

  if (!isAuthenticated) {
    return (
      <LoginPage
        onAuthenticated={() => setIsAuthenticated(true)}
      />
    )
  }

  if (selectedCaseId !== null) {
    return (
      <InvestigationPage
        caseId={selectedCaseId}
        onBack={() => setSelectedCaseId(null)}
      />
    )
  }

  return (
    <DashboardPage
      onLogout={handleLogout}
      onOpenCase={(id) => setSelectedCaseId(id)}
    />
  )
}

export default App

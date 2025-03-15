import { useState } from 'react'
import './App.css'
import { BrowserRouter as Router, Routes, Route } from "react-router-dom"
import Home from './pages/Home'
import Warning from './components/Warning'
import Agent from './pages/Agent'

function App() {
  const [wallet, setWallet] = useState(null)
  const [agent, setAgent] = useState(null)
  const [warningMessage, setWarningMessage] = useState('');

  const closeWarning = () => setWarningMessage('');

  return (
    <Router>
      <Warning message={warningMessage} onClose={closeWarning} />
      <Routes>
        <Route path="/"
          element={
            <Home
              title={"Runic-Jungle"}
              agent={agent}
              setAgent={setAgent}
              wallet={wallet}
              setWallet={setWallet}
              setWarningMessage={setWarningMessage}
            />
          }
        />
        <Route path="/agent"
          element={
            <Agent
              agent={agent}
              setAgent={setAgent}
              wallet={wallet}
              setWallet={setWallet}
              setWarningMessage={setWarningMessage}
            />
          }
        />
      </Routes>
    </Router >
  )
}

export default App

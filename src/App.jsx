import { useState } from 'react'
import './App.css'
import { BrowserRouter as Router, Routes, Route } from "react-router-dom"
import Home from './pages/Home'

function App() {
  const [wallet, setWallet] = useState(null)
  const [agent, setAgent] = useState(null)

  return (
    <Router>
      <Routes>
        <Route path="/" element={<Home title={"Runic-Agent"} setAgent={setAgent} wallet={wallet} setWallet={setWallet} />} />
      </Routes>
    </Router>
  )
}

export default App

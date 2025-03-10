import { useState } from 'react'
import './App.css'
import Navbar from "./components/Navbar";

function App() {
  const [wallet, setWallet] = useState(null);

  return (
    <>
      <Navbar title={"Runic-Agent"} wallet={wallet} setWallet={setWallet}/>
    </>
  )
}

export default App

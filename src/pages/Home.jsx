import "./../styles/Home.css"
import React, { useState, useEffect } from "react";
import CreateAgent from "../components/CreateAgent";
import Navbar from "../components/Navbar";
import { backend } from "../declarations/backend";
import { Link } from "react-router-dom"

export default function Home({
  title,
  bitcoinAddress,
  setBitcoinAddress,
  bitcoinBalance,
  setBitcoinBalance,
  agent,
  setAgent,
  wallet,
  setWallet,
  setWarningMessage,
}) {
  const [agents, setAgents] = useState([]);

  // Function to fetch the agents list from the backend
  const fetchAgents = async () => {
    try {
      const agentList = await backend.get_agents();
      setAgents(agentList);
    } catch (error) {
      setAgents([])
      console.error("Error fetching agents:", error);
    }
  };

  // Fetch agents on mount and then refresh every 60 seconds
  useEffect(() => {
    fetchAgents();
    const interval = setInterval(fetchAgents, 60000); // 60,000 ms = 1 minute
    return () => clearInterval(interval);
  }, []);

  return (
    <>
      <Navbar
        title={title}
        bitcoinAddress={bitcoinAddress}
        setBitcoinAddress={setBitcoinAddress}
        bitcoinBalance={bitcoinBalance}
        setBitcoinBalance={setBitcoinBalance}
        agent={agent}
        setAgent={setAgent}
        wallet={wallet}
        setWallet={setWallet}
        setWarningMessage={setWarningMessage}
      />
      <main>
        <p>Welcome to Bitcoin Agent Jungle Powered by Internet Computer</p>
        <CreateAgent wallet={wallet} />

        <section className="agent-list">
          {agents.map(([agentId, details]) => (
            <Link
              key={agentId}
              to={`/agent?id=${agentId}`}
              className="agent-item"
            >
              <h2>{details.agent_name}</h2>
              <p>{details.description}</p>
            </Link>
          ))}
        </section>
      </main>

    </>
  );
}

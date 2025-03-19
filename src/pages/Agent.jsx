import { backend } from "./../declarations/backend";
import "./../styles/Agent.css";
import Navbar from "./../components/Navbar";
import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import Market from "../components/Market";
import Jackpot from "../components/Jackpot";
import Chatbox from "../components/Chatbox"

export default function Agent({
  agent,
  bitcoinAddress,
  setBitcoinAddress,
  bitcoinBalance,
  setBitcoinBalance,
  setAgent,
  wallet,
  setWallet,
  setWarningMessage,
}) {
  const location = useLocation();
  const [agentData, setAgentData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [selectedTab, setSelectedTab] = useState("market");

  const query = new URLSearchParams(location.search);
  const idParam = query.get("id");
  const id = idParam ? Number(idParam) : null;

  useEffect(() => {
    if (id === null || isNaN(id)) {
      setError("Invalid or missing id parameter");
      setLoading(false);
      return;
    }
    const agentByArg = { Id: id };

    backend
      .get_agent_of(agentByArg)
      .then((result) => {
        if (result) {
          console.log("from_agent_page");
          console.log(result[0]);
          setAgentData(result[0]);
        } else {
          setError("No agent data found.");
        }
      })
      .catch((err) => {
        console.error("Error fetching agent data:", err);
        setError("Failed to load agent data.");
      })
      .finally(() => setLoading(false));
  }, [id]);

  // Convert BigInt to Number explicitly before performing arithmetic
  const marketCapInBTC = agentData
    ? (Number(agentData.market_cap) / 1e8).toFixed(8)
    : null;

  const renderTabContent = () => {
    if (!agentData) return null;
    switch (selectedTab) {
      case "market":
        return (
          <Market agent_id={id} symbol={agentData.ticker} wallet={wallet} setWarningMessage={setWarningMessage} />
        );
      case "jackpot":
        return (
          <Jackpot agent_id={id} wallet={wallet} current_winner={agentData.current_winner} current_prize_pool={agentData.current_prize_pool} setWarningMessage={setWarningMessage} />
        );
      case "chat":
        return (
          <Chatbox wallet={wallet} agent={id} />
        );
      default:
        return null;
    }
  };

  if (loading) return <div>Loading agent data...</div>;
  if (error) return <div className="error">{error}</div>;

  return (
    <div className="agent-page">
      <Navbar
        title={agentData.agent_name}
        agent={agent}
        bitcoinAddress={bitcoinAddress}
        setBitcoinAddress={setBitcoinAddress}
        bitcoinBalance={bitcoinBalance}
        setBitcoinBalance={setBitcoinBalance}
        setAgent={setAgent}
        wallet={wallet}
        setWallet={setWallet}
        setWarningMessage={setWarningMessage}
      />
      <div className="agent-details">
        <h1>{agentData.agent_name}</h1>
        <p>{agentData.description}</p>
        <div className="agent-extra-info">
          <p>
            <strong>Market Cap:</strong> ₿ {marketCapInBTC}
          </p>
          <p>
            <strong>Ticker:</strong> {String.fromCharCode(agentData.ticker)}
          </p>
          <p>
            <strong>Holders:</strong> {agentData.holders}
          </p>
        </div>
        {agentData.logo && (
          <img
            src={agentData.logo}
            alt={`${agentData.agent_name} logo`}
            className="agent-logo"
          />
        )}
      </div>
      <div className="tabs">
        <button
          className={`tab  ${selectedTab === "market" ? "active" : ""}`}
          onClick={() => setSelectedTab("market")}
        >
          Market
        </button>
        <button
          className={`tab ${selectedTab === "jackpot" ? "active" : ""}`}
          onClick={() => setSelectedTab("jackpot")}
        >
          Hit The Jackpot
        </button>
        <button
          className={`tab  ${selectedTab === "chat" ? "active" : ""}`}
          onClick={() => setSelectedTab("chat")}
        >
          Chat with Bot
        </button>
      </div>
      <div className="tab-container">{renderTabContent()}</div>
    </div>
  );
}

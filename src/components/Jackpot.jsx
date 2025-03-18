import React, { useState, useEffect } from 'react';
import { HttpAgent } from "@dfinity/agent";
import { createActor, canisterId } from "../declarations/backend/index";
import './../styles/Jackpot.css';

export default function Jackpot({
  agent_id,
  wallet,
  current_winner,
  current_prize_pool,
  setWarningMessage,
}) {
  const [inputText, setInputText] = useState("");
  const [backend, setBackend] = useState(null);

  useEffect(() => {
    async function initBackend() {
      try {
        const agent = await HttpAgent.create({ identity: wallet });
        const backendActor = createActor(canisterId, { agent });
        setBackend(backendActor);
      } catch (error) {
        setWarningMessage("Failed to initialize backend: " + error.message);
      }
    }
    initBackend();
  }, [wallet, setWarningMessage]);

  const [btc, rune] = current_prize_pool;
  const formattedBTC = (Number(btc) / Math.pow(10, 8)).toFixed(8);
  const formattedRune = (Number(rune) / Math.pow(10, 3)).toFixed(3);

  const handleSubmit = async () => {
    if (!backend) {
      setWarningMessage("Backend is not initialized yet.");
      return;
    }
    try {
      const arg = { id: { Id: agent_id }, message: inputText };
      const response = await backend.lucky_draw(arg);
      setWarningMessage(response);
    } catch (error) {
      setWarningMessage("Error: " + error.message);
    }
  };

  return (
    <div className="jackpot">
      <h1 className="title">Bait the Bot</h1>

      {current_winner[0] ? (
        <div className="winner-message">
          Jackpot already claimed by user: {current_winner}
        </div>
      ) : (
        <>
          <div className="prize-pool">
            <span>Bitcoin Prize: {formattedBTC}</span>
            <span>Rune Token Prize: {formattedRune}</span>
          </div>

          <div className="input-container">
            <input
              type="text"
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              placeholder="Enter your message..."
              className="text-input"
            />
            <button onClick={handleSubmit} className="submit-button">
              Submit
            </button>
          </div>
        </>
      )}
    </div>
  );
}

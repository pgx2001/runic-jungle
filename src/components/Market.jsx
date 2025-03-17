import React, { useState } from "react";
import { HttpAgent } from "@dfinity/agent";
import { createActor, canisterId } from "../declarations/backend";
import "./../styles/Market.css";

export default function Market({ agent_id, symbol, wallet, setWarningMessage }) {
  const [selectedTab, setSelectedTab] = useState("buy");
  const [amount, setAmount] = useState("");
  const [loading, setLoading] = useState(false);

  const handleBuy = async () => {
    try {
      setLoading(true);
      const agentInstance = await HttpAgent.create({ identity: wallet });
      const backend = createActor(canisterId, { agent: agentInstance });
      const buyArgs = {
        id: { Id: agent_id },
        amount_out_min: 0,
        buy_exact_in: BigInt(amount),
      };
      const result = await backend.buy(buyArgs);
      setWarningMessage(`You received ${result} ${symbol}.`);
    } catch (err) {
      console.error(err);
      setWarningMessage("Buy transaction failed.");
    } finally {
      setLoading(false);
    }
  };

  const handleSell = async () => {
    try {
      setLoading(true);
      const agentInstance = await HttpAgent.create({ identity: wallet });
      const backend = createActor(canisterId, { agent: agentInstance });
      const sellArgs = {
        id: { Id: agent_id },
        token_amount: Number(amount),
        amount_collateral_min: 0,
      };
      const result = await backend.sell(sellArgs);
      setWarningMessage(`You received ${result} ${String.fromCharCode(symbol)}.`);
    } catch (err) {
      console.error(err);
      setWarningMessage("Sell transaction failed.");
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    if (selectedTab === "buy") {
      handleBuy();
    } else {
      handleSell();
    }
  };

  return (
    <div className="market">
      <h1 className="title">Market</h1>
      <div className="tabs">
        <button
          className={selectedTab === "buy" ? "tab active" : "tab"}
          onClick={() => setSelectedTab("buy")}
        >
          Buy
        </button>
        <button
          className={selectedTab === "sell" ? "tab active" : "tab"}
          onClick={() => setSelectedTab("sell")}
        >
          Sell
        </button>
      </div>
      <form onSubmit={handleSubmit} className="market-form">
        <label>{selectedTab === "buy" ? "Buy Amount:" : "Sell Amount:"}</label>
        <input
          type="number"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="Enter amount"
          required
        />
        <button type="submit" disabled={loading}>
          {loading ? "Processing..." : selectedTab === "buy" ? "Buy" : "Sell"}
        </button>
      </form>
    </div>
  );
}

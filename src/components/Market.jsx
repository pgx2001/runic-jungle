import React, { useState } from "react";
import { HttpAgent } from "@dfinity/agent";
import { createActor, canisterId } from "../declarations/backend";
import "./../styles/Market.css";

export default function Market({ agent_id, wallet, setWarningMessage }) {
  const [selectedTab, setSelectedTab] = useState("buy");
  const [amount, setAmount] = useState("");
  const [loading, setLoading] = useState(false);

  const handleBuy = async () => {
    try {
      setLoading(true);
      const agentInstance = await HttpAgent.create({ identity: wallet });
      const backend = createActor(canisterId, { agent: agentInstance });
      // Convert BTC (with 8 decimals) into satoshis
      const btcAmount = parseFloat(amount);
      const btcAmountInSatoshis = BigInt(Math.floor(btcAmount * 10 ** 8));
      const buyArgs = {
        id: { Id: agent_id },
        amount_out_min: 0,
        buy_exact_in: btcAmountInSatoshis,
      };
      const result = await backend.buy(buyArgs);
      // Convert token amount (assumed to be in integer form with 3 decimals) back to human readable value
      const tokenAmountHuman = Number(result) / 10 ** 3;
      setWarningMessage(`You received ${tokenAmountHuman} tokens.`);
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
      // Convert token amount (with 3 decimals) into backend integer representation
      const tokenAmount = parseFloat(amount);
      const tokenAmountInUnits = BigInt(Math.floor(tokenAmount * 10 ** 3));
      const sellArgs = {
        id: { Id: agent_id },
        token_amount: tokenAmountInUnits,
        amount_collateral_min: 0,
      };
      const result = await backend.sell(sellArgs);
      // Convert returned BTC amount (in satoshis) back to BTC
      const btcReceived = Number(result) / 10 ** 8;
      setWarningMessage(`You received ${btcReceived} BTC.`);
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
        <label>
          {selectedTab === "buy" ? "Enter BTC amount:" : "Enter Token amount:"}
        </label>
        <input
          type="number"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="Enter amount"
          required
        />
        <button
          type="submit"
          disabled={loading}
          className={selectedTab === "buy" ? "btn-buy" : "btn-sell"}
        >
          {loading
            ? "Processing..."
            : selectedTab === "buy"
              ? "Buy with BTC"
              : "Sell Tokens"}
        </button>
      </form>
    </div>
  );
}

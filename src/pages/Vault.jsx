import { useEffect, useState } from "react";
import { createActor, canisterId } from "./../declarations/backend";
import { HttpAgent } from "@dfinity/agent";
import "./../styles/Vault.css"
import Navbar from "../components/Navbar";

export default function Vault({
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
  const [balances, setBalances] = useState({});
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchBalances = async () => {
      try {
        const agentInstance = await HttpAgent.create({ identity: wallet });
        const backend = createActor(canisterId, { agent: agentInstance });
        const result = await backend.get_balances();
        console.log(result);
        setBalances(result);
      } catch (error) {
        console.error("Error fetching balances:", error);
      } finally {
        setLoading(false);
      }
    };

    fetchBalances();
  }, [wallet]);

  const formatBalance = (name, balance) => {
    const lowerName = name.toLowerCase();
    const numericBalance = parseFloat(balance); // Ensure it's a number

    if (isNaN(numericBalance)) {
      return "Invalid balance"; // Handle invalid balances gracefully
    }

    return lowerName === "bitcoin" ? numericBalance.toFixed(8) : numericBalance.toFixed(3);
  };

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
      <div className="vault-container">

        <h1>Vault</h1>
        {loading ? (
          <p className="loading">Loading balances...</p>
        ) : (
          <table className="balance-table">
            <thead>
              <tr>
                <th>Token</th>
                <th>Balance</th>
              </tr>
            </thead>
            <tbody>
              {Object.entries(balances).map(([name, balance]) => (
                <tr key={name}>
                  <td>{name}</td>
                  <td>{formatBalance(name, balance)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </>
  );
}

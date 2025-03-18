import "./../styles/Navbar.css";
import { identityProvider } from "./../config";
import { AuthClient } from "@dfinity/auth-client";
import { Link, } from "react-router-dom";
import { createActor, canisterId } from "./../declarations/backend";
import { HttpAgent } from "@dfinity/agent";
import { useCallback } from "react";

export default function Navbar({
  title,
  bitcoinAddress,
  setBitcoinAddress,
  bitcoinBalance,
  setBitcoinBalance,
  wallet,
  agent,
  setAgent,
  setWallet,
  setWarningMessage
}) {
  const fetchBitcoinData = useCallback(async (identity) => {
    try {
      const agent = await HttpAgent.create({ identity });
      const backend = createActor(canisterId, { agent });

      console.log(title)
      const bitcoin_address = await backend.get_deposit_address();
      console.log("bitcoin adr")
      console.log(bitcoin_address)
      const bitcoin_balance = await backend.get_bitcoin_balance();

      setBitcoinAddress(bitcoin_address);
      setBitcoinBalance(Number(bitcoin_balance) / 10 ** 8);
    } catch (error) {
      console.error("Error fetching Bitcoin data:", error);
      setWarningMessage("Failed to fetch Bitcoin data");
    }
  }, [title, setBitcoinAddress, setBitcoinBalance, setWarningMessage]);

  const handleClick = async () => {
    try {
      const authClient = await AuthClient.create();

      if (wallet) {
        await authClient.logout();
        setWallet(null);
        setBitcoinAddress(null);
        setBitcoinBalance(null);
        return;
      }

      await authClient.login({ identityProvider });
      const identity = authClient.getIdentity();
      const principal = identity.getPrincipal().toString();

      if (principal === "2vxsx-fae") {
        throw new Error("Login failed: temporary anonymous identity returned.");
      }

      setWallet(identity);
      setWarningMessage(null); // Clear any previous error

      // Immediately fetch Bitcoin data after login
      fetchBitcoinData(identity);
    } catch (error) {
      setWarningMessage("Failed to connect wallet");
      console.error("Authentication error:", error);
    }
  };

  return (
    <nav>
      <Link className="title" to="/">{title}</Link>
      {agent && (
        <Link className="home-btn" onClick={() => setAgent(null)} to="/">
          Create your own AI agent
        </Link>
      )}
      <div className="nav-right">
        {wallet && bitcoinAddress && (
          <span className="bitcoin-info">
            {bitcoinBalance !== null ? `Balance: ${bitcoinBalance}₿` : "Loading..."} | Address: {bitcoinAddress}
          </span>
        )}
        {wallet && <Link className="vault-btn" to="/vault">Vault</Link>}
        <button className="wallet-btn" onClick={handleClick}>
          {wallet ? "[ logout ]" : "[ connect ]"}
        </button>
      </div>
    </nav>
  );
}

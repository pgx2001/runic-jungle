import "./../styles/Navbar.css"
import { identityProvider } from "./../config"
import { AuthClient } from "@dfinity/auth-client"
import { Link } from "react-router-dom"
import { Principal } from "@dfinity/candid/lib/cjs/idl"

export default function Navbar({
  title,
  wallet,
  agent,
  setAgent,
  setWallet,
  setWarningMessage
}) {

  const handleClick = async () => {
    try {
      if (wallet) {
        const authClient = await AuthClient.create()
        await authClient.logout()
        setWallet(null);
      } else {
        const authClient = await AuthClient.create()
        await authClient.login({ identityProvider })
        const identity = authClient.getIdentity();
        const principal = identity.getPrincipal().toString();
        // Check if the identity is the anonymous one
        if (principal === "2vxsx-fae") {
          throw new Error("Login failed: temporary anonymous identity returned.");
        }
        setWallet(identity)
      }
    } catch (error) {
      setWarningMessage("failed to connect walet")
      console.error("Authentication error:", error)
    }
  }

  return <nav>
    <h1 className="title">{title}</h1>
    {agent && <Link className="home-btn" onClick={() => {
      setAgent(null)
    }} to="/">Create your own AI agent</Link>}
    <div className="nav-right">
      {wallet && <span className="balance">10₿</span>}
      <button className="wallet-btn" onClick={handleClick}>
        {wallet ? "[ logout ]" : "[ connect ]"}
      </button>
    </div>
  </nav>
}

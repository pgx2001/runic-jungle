import CreateAgent from "../components/CreateAgent";
import Navbar from "../components/Navbar";

export default function Home({
  title,
  agent,
  setAgent,
  wallet,
  setWallet,
  setWarningMessage
}) {
  return <>
    <Navbar title={title} agent={agent} setAgent={setAgent} wallet={wallet} setWallet={setWallet} setWarningMessage={setWarningMessage} />
    <main>
      <p>Welcome to Bitcoin Agent Jungle Powered by Internet Computer</p>
      <CreateAgent wallet={wallet} />
    </main>
  </>
}

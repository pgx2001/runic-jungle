import CreateAgent from "../components/CreateAgent";
import Navbar from "../components/Navbar";

export default function Home({
  title,
  setAgent,
  wallet,
  setWallet
}) {
  return <>
    <Navbar title={title} setAgent={setAgent} wallet={wallet} setWallet={setWallet} />
    <main>
      <p>Welcome to Bitcoin Agent Jungle Powered by Internet Computer</p>
      <CreateAgent wallet={wallet} />
    </main>
  </>
}

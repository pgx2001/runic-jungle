import Navbar from "./../components/Navbar"
import { useEffect, useState } from 'react';
import { useLocation } from 'react-router-dom';

export default function Agent({
  agent,
  setAgent,
  wallet,
  setWallet,
  setWarningMessage
}) {
  const location = useLocation();
  const [agentData, setAgentData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const query = new URLSearchParams(location.search);
  const idParam = query.get('id');
  const id = idParam ? Number(idParam) : null;

  useEffect(() => {
    if (id === null || isNaN(id)) {
      setError('Invalid or missing id parameter');
      setLoading(false);
      return;
    }
    setLoading(false)
    console.log("id:", id)

    // Create the variant argument with the number value.
    // const agentByArg = { Id: id };


    // Call the smart contract method
    /* backend.get_agent_of(agentByArg)
      .then((result) => {
        setAgentData(result); // result is an Option
      })
      .catch((err) => {
        console.error('Error fetching agent data:', err);
        setError('Failed to load agent data.');
      })
      .finally(() => setLoading(false)); */
  }, [id, /*backend*/]);

  if (loading) return <div>Loading agent data...</div>;
  if (error) return <div>{error}</div>;


  return <div>
    <Navbar title={"gooad"} agent={agent} setAgent={setAgent} wallet={wallet} setWallet={setWallet} setWarningMessage={setWarningMessage} />
    {agentData ? (
      <div>
        <p><strong>Agent Name:</strong> {"gooad"}</p>
        <p><strong>Description:</strong> {"lorem*10"}</p>
        <p><strong>Market Cap:</strong> {"100btc"}</p>
      </div>
    ) : (
      <p>No agent data found.</p>
    )}
  </div>
}

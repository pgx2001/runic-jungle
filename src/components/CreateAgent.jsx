import { HttpAgent } from "@dfinity/agent";
import { createActor, canisterId } from "./../declarations/backend/index"

export default function CreateAgent({ wallet }) {
  const create_agent = async () => {
    const agent = await HttpAgent.create({ identity: wallet });
    const backend = createActor(canisterId, { agent });
    backend.create_agent({})
  }

  return <></>
}

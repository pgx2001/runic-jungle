import { useEffect, useRef, useState } from 'react';
import { createActor, canisterId } from '../declarations/backend';
import { HttpAgent } from '@dfinity/agent';
import "./../styles/Chatbox.css"

export default function Chatbox({ wallet, agent }) {
  const [sessionId, setSessionId] = useState(null);
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState('');
  const chatEndRef = useRef(null);

  useEffect(() => {
    console.log("called to create session id")
    const initializeChat = async () => {
      try {
        const agentInstance = await HttpAgent.create({ identity: wallet });
        const backend = createActor(canisterId, { agent: agentInstance });

        console.log("calling for new session id")
        const newSessionId = await backend.create_chat_session({ 'Id': agent });
        console.log("session id:", newSessionId)
        setSessionId(newSessionId);
      } catch (error) {
        console.error('Error initializing chat:', error);
      }
    };

    if (wallet) {
      initializeChat();
    }
  }, [wallet, agent]);

  const sendMessage = async () => {
    if (!input.trim() || sessionId === null) return;

    try {
      const agentInstance = await HttpAgent.create({ identity: wallet });
      const backend = createActor(canisterId, { agent: agentInstance });

      console.log("calling chat")
      const response = await backend.chat({
        agent: { 'Id': agent },
        session_id: sessionId,
        message: input,
      });
      console.log("response: ", response)

      setMessages((prev) => [...prev, { sender: 'You', text: input }, { sender: 'AI', text: response }]);
      setInput('');
    } catch (error) {
      console.error('Error sending message:', error);
    }
  };

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  return (
    <div className="chatbox">
      <h1 className="title">Chat with Bot</h1>
      <div className="messages">
        {messages.map((msg, index) => (
          <div key={index} className={`message ${msg.sender}`}>{msg.sender}: {msg.text}</div>
        ))}
        <div ref={chatEndRef}></div>
      </div>
      <input
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={(e) => e.key === 'Enter' && sendMessage()}
        placeholder="Type a message..."
      />
      <button onClick={() => { sendMessage() }}>Send</button>
    </div>
  );
}

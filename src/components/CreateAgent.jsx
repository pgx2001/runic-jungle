import React, { useState } from "react";
import { HttpAgent } from "@dfinity/agent";
import { createActor, canisterId } from "../declarations/backend/index";
import "./../styles/CreateAgent.css";

export default function CreateAgent({ wallet, setWarningMessage }) {
  const [isOpen, setIsOpen] = useState(false);
  const [formData, setFormData] = useState({
    ticker: "",
    twitter: "",
    logo: "",
    name: "",
    description: "",
    website: "",
    discord: "",
    openchat: "",
  });

  const handleChange = (e) => {
    setFormData({ ...formData, [e.target.name]: e.target.value });
  };

  const handleFileChange = (e) => {
    const file = e.target.files[0];
    if (file) {
      if (file.size > 1.2 * 1024 * 1024) {
        setWarningMessage("File size must be under 1.2MB");
        return;
      }
      setWarningMessage(null);
      const reader = new FileReader();
      reader.onloadend = () => {
        setFormData({ ...formData, logo: reader.result });
      };
      reader.readAsDataURL(file);
    }
  };

  const create_agent = async () => {
    try {
      const agent = await HttpAgent.create({ identity: wallet });
      const backend = createActor(canisterId, { agent });
      await backend.create_agent({
        ticker: formData.ticker ? [Number(formData.ticker)] : [],
        twitter: formData.twitter ? [formData.twitter] : [],
        logo: formData.logo ? [formData.logo] : [],
        name: formData.name,
        description: formData.description,
        website: formData.website ? [formData.website] : [],
        discord: formData.discord ? [formData.discord] : [],
        openchat: formData.openchat ? [formData.openchat] : [],
      });
      setIsOpen(false);
    } catch (error) {
      setIsOpen(false);
      const errorString = error.toString();

      // Use a regex to extract the reject message inside single quotes after "with message:"
      const match = errorString.match(/with message:\s*'([^']+)'/);

      // If a match is found, use it; otherwise fallback to the entire error string
      const rejectMessage = match ? match[1] : errorString;

      setWarningMessage(rejectMessage);
      console.error("Extracted reject message:", rejectMessage);
    }
  };

  return (
    <div>
      <button className="open-modal-btn" onClick={() => setIsOpen(true)}>Create Agent</button>
      {isOpen && (
        <div className="modal-overlay">
          <div className="modal">
            <h2>Create Agent</h2>
            <input type="text" name="name" placeholder="Name" onChange={handleChange} required />
            <input type="text" name="ticker" placeholder="Ticker (optional)" onChange={handleChange} />
            <input type="text" name="twitter" placeholder="Twitter (optional)" onChange={handleChange} />
            <input type="text" name="website" placeholder="Website (optional)" onChange={handleChange} />
            <input type="text" name="discord" placeholder="Discord (optional)" onChange={handleChange} />
            <input type="text" name="openchat" placeholder="OpenChat (optional)" onChange={handleChange} />
            <textarea name="description" placeholder="Description" onChange={handleChange} required></textarea>
            <input type="file" accept="image/*" onChange={handleFileChange} />
            {formData.logo && <img src={formData.logo} alt="Logo Preview" className="logo-preview" />}
            <div className="modal-actions">
              <button className="cancel-btn" onClick={() => setIsOpen(false)}>Cancel</button>
              <button className="confirm-btn" onClick={create_agent}>Confirm</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

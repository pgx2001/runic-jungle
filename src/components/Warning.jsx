import './../styles/Warning.css';

export default function Warning({ message, onClose }) {
  if (!message) return null;

  return (
    <div className="overlay">
      <div className="warning-container">
        <p className="warning-message">{message}</p>
        <button className="close-button" onClick={onClose}>
          &times;
        </button>
      </div>
    </div>
  );
};


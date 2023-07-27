import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen }  from "@tauri-apps/api/event";
import { Letter } from 'react-letter';
import { extract } from 'letterparser';
import Image from "next/image";
import reactLogo from "../assets/react.svg";
import tauriLogo from "../assets/tauri.svg";
import nextLogo from "../assets/next.svg";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const client_secret = "YOUR_CLIENT_SECRET"
  const [client_id, set_client_id] = useState([]);
  const [loading, setLoading] = useState(true);
  const [rd, setRd] = useState([]);
  const [mails, setMails] = useState({});
  const [error, setError] = useState(null);


  const unlisten = listen('redirect-data', (event) => {
    console.log(event.payload)
    // event.event is the event name (useful if you want to use a single callback fn for multiple event types)
    // event.payload is the payload object
  })
  const unlisten_mails = listen('mail-content', (event) => {
    console.log(event.payload)
  
    
    const { html, text } = extract(event.payload);
    setMails({html,text});   
    // event.event is the event name (useful if you want to use a single callback fn for multiple event types)
    // event.payload is the payload object
  })

  function getOauthUrl(clientID: string) {
    return `https://accounts.google.com/o/oauth2/v2/auth?client_id=${clientID}&redirect_uri=http://localhost:8000&access_type=offline&response_type=code&scope=https://mail.google.com/`;
  }
//obtain client_id from backend with invoke command
useEffect(() => {
  const fetchData = async () => {
    try {
      const response = await invoke('get_client_id')
        console.log(response)
    set_client_id(response)
    } catch (error) {
      setError(error.message);
    } finally {
      setLoading(false);
    }
  };

  fetchData();
}, []);


  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>

      <div className="row">
      </div>
      <div>
      {loading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      {!loading && !error && (
        <p>
          {getOauthUrl(client_id)}
        </p>
      )}
    </div>
      <div className="row">
        <div>
          <input
            id="greet-input"
            onChange={(e) => setName(e.currentTarget.value)}
            placeholder="Enter a name..."
          />
          <a href={getOauthUrl(client_id)} target="_blank">
            Greet
          </a>
          <Letter html={mails.html} text={mails.text} />
        </div>
      </div>

      <p>{greetMsg}</p>
    </div>
  );
}

export default App;

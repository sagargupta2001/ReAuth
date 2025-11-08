import { useEffect, useState } from "react";

const HelloWorldPage = () => {
  const [message, setMessage] = useState("Loading message from backend...");

  useEffect(() => {
    // This function is called when the components mounts
    const fetchMessage = async () => {
      try {
        // Make the HTTP request to the Core's proxy endpoint
        const response = await fetch("/api/plugins/hello-world/say-hello");
        const data = await response.json();
        setMessage(data.message);
      } catch (error) {
        setMessage("Failed to load message from backend.");
        console.error(error);
      }
    };

    fetchMessage().catch((e) => console.log(e));
  }, []);

  return (
    <div
      style={{ padding: "20px", border: "1px solid #ddd", borderRadius: "8px" }}
    >
      <h1>Hello World Plugin</h1>
      <p>This React component was loaded dynamically at runtime.</p>
      <p style={{ marginTop: "20px", fontWeight: "bold" }}>
        Message from backend: "{message}"
      </p>
    </div>
  );
};

export default HelloWorldPage;

import { useState, useCallback } from 'react';

function App() {
  const [socket, setSocket] = useState(null);
  const [connected, setConnected] = useState(false);
  const [messages, setMessages] = useState([]);
  const [inputMessage, setInputMessage] = useState('');

  const connectWebSocket = useCallback(() => {
    if (socket) {
      socket.close();
      setSocket(null);
      setConnected(false);
      return;
    }

    const ws = new WebSocket('ws://127.0.0.1:8008');

    ws.onopen = () => {
      console.log('Connected to WebSocket');
      setConnected(true);
    };

    ws.onmessage = (event) => {
      setMessages(prev => [...prev, { text: event.data, type: 'received' }]);
    };

    ws.onclose = () => {
      console.log('Disconnected from WebSocket');
      setConnected(false);
      setSocket(null);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      setConnected(false);
      setSocket(null);
    };

    setSocket(ws);
  }, [socket]);

  const sendMessage = useCallback(() => {
    if (socket && inputMessage) {
      socket.send(inputMessage);
      setMessages(prev => [...prev, { text: inputMessage, type: 'sent' }]);
      setInputMessage('');
    }
  }, [socket, inputMessage]);

  return (
    <div className="min-h-screen bg-gray-100 p-8">
      <div className="max-w-md mx-auto bg-white rounded-xl shadow-md overflow-hidden md:max-w-2xl p-6">
        <h1 className="text-2xl font-bold mb-4">WebSocket Echo Demo</h1>
        
        <button
          onClick={connectWebSocket}
          className={`px-4 py-2 rounded-md ${
            connected 
              ? 'bg-red-500 hover:bg-red-600 text-white'
              : 'bg-blue-500 hover:bg-blue-600 text-white'
          } transition-colors`}
        >
          {connected ? 'Disconnect' : 'Connect to WebSocket'}
        </button>

        <div className="mt-4">
          <p className="text-gray-600">
            Status: {connected ? 'Connected' : 'Disconnected'}
          </p>
        </div>

        {connected && (
          <div className="mt-4 space-y-4">
            <div className="flex space-x-2">
              <input
                type="text"
                value={inputMessage}
                onChange={(e) => setInputMessage(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
                placeholder="Type a message to echo..."
                className="flex-1 px-4 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <button
                onClick={sendMessage}
                className="px-4 py-2 bg-green-500 text-white rounded-md hover:bg-green-600 transition-colors"
              >
                Send
              </button>
            </div>
          </div>
        )}

        {messages.length > 0 && (
          <div className="mt-4">
            <h2 className="text-lg font-semibold mb-2">Messages:</h2>
            <div className="space-y-2">
              {messages.map((msg, index) => (
                <div 
                  key={index} 
                  className={`p-2 rounded ${
                    msg.type === 'sent' 
                      ? 'bg-blue-100 text-blue-800 ml-8'
                      : 'bg-gray-100 text-gray-800 mr-8'
                  }`}
                >
                  {msg.type === 'sent' ? '→ ' : '← '}{msg.text}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;

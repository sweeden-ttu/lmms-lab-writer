let localSocket = null;

function connectToLocalTauri() {
  // Connect to the local Tauri backend server
  localSocket = new WebSocket('ws://localhost:3030/adk-eval');

  localSocket.onopen = () => {
    console.log('[adk-rust] Connected to local Tauri backend');
  };

  localSocket.onmessage = (event) => {
    console.log('[adk-rust] Received from Tauri:', event.data);
    // Could forward requests to content scripts
  };

  localSocket.onclose = () => {
    console.log('[adk-rust] Disconnected, retrying in 5s...');
    setTimeout(connectToLocalTauri, 5000);
  };
}

// Initialize connection
connectToLocalTauri();

// Listen for messages from content scripts
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === 'evaluate_course_content') {
    console.log('Received course content from page:', request.payload);
    
    // Send to local Tauri adk-rust backend
    if (localSocket && localSocket.readyState === WebSocket.OPEN) {
      localSocket.send(JSON.stringify({
        type: 'evaluation_payload',
        data: request.payload
      }));
      sendResponse({ status: 'sent_to_adk' });
    } else {
      sendResponse({ status: 'error', message: 'Tauri backend disconnected' });
    }
  }

  return true;
});

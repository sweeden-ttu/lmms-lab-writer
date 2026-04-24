document.addEventListener('DOMContentLoaded', () => {
  const importBtn = document.getElementById('importBtn');
  const scheduleSelect = document.getElementById('scheduleSelect');
  const statusEl = document.getElementById('status');

  importBtn.addEventListener('click', async () => {
    const schedule = scheduleSelect.value;
    
    // In the future, this is where we would use chrome.alarms for scheduling
    if (schedule !== 'one-time') {
      statusEl.innerText = `Scheduled: ${schedule} crawl (Simulation)`;
      // We still run it once now for testing purposes
    } else {
      statusEl.innerText = "Starting one-time full crawl...";
    }
    
    importBtn.disabled = true;
    importBtn.innerText = "Crawling...";

    // Query the active tab
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab) {
      statusEl.innerText = "Error: No active tab found.";
      return;
    }

    // Send message to the content script to start the nested crawl
    chrome.tabs.sendMessage(tab.id, { action: 'start_nested_crawl' }, (response) => {
      if (chrome.runtime.lastError) {
        console.error(chrome.runtime.lastError);
        statusEl.innerText = "Error: Please refresh the page first.";
        importBtn.disabled = false;
        importBtn.innerText = "Import...";
        return;
      }
      
      if (response && response.status === 'started') {
        statusEl.innerText = "Crawl started in background.";
        setTimeout(() => window.close(), 2000); // Close popup after a bit
      } else {
        statusEl.innerText = "Error starting crawl.";
        importBtn.disabled = false;
        importBtn.innerText = "Import...";
      }
    });
  });

  // Placeholder actions for the other buttons
  document.getElementById('settingsBtn').addEventListener('click', () => {
    statusEl.innerText = "Settings clicked (Not implemented)";
  });
  
  document.getElementById('logsBtn').addEventListener('click', () => {
    statusEl.innerText = "Logs clicked (Not implemented)";
  });
});

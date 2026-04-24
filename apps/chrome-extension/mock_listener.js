const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 3222;
const OUTPUT_FILE = '/tmp/course_payload.json';

const server = http.createServer((req, res) => {
  // CORS headers for the Chrome Extension
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  if (req.method === 'POST' && req.url === '/upload') {
    console.log(`[mock_listener] Receiving stream. Piping directly back as a downloaded file...`);
    
    res.writeHead(200, {
      'Content-Type': 'application/json',
      'Content-Disposition': 'attachment; filename="course_payload.json"'
    });
    
    // Pipe the incoming request stream directly into the response stream
    req.pipe(res);

    req.on('end', () => {
      console.log('[mock_listener] Stream finished and piped back to client.');
    });

    req.on('error', (err) => {
      console.error('[mock_listener] Error receiving stream:', err);
      if (!res.headersSent) res.writeHead(500);
      res.end();
    });
  } else {
    res.writeHead(404);
    res.end('Not Found');
  }
});

server.listen(PORT, () => {
  console.log(`Mock listener actively running on http://localhost:${PORT}`);
  console.log(`Ready to pipe incoming streams back as downloads.`);
});

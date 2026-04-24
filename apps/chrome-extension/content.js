// Comprehensive reliable scraping of the course content
function extractCourseData(doc = document, currentUrl = window.location.href) {
  // 1. Full visible text: the most reliable way to get what a human actually sees on the page
  const fullText = doc.body ? doc.body.textContent : '';
    
  // 2. Grab all headers to understand the document structure
  const headers = Array.from(doc.querySelectorAll('h1, h2, h3, h4, h5, h6'))
    .map(el => el.textContent.trim())
    .filter(text => text.length > 0);

  // 3. Grab all paragraph-like text without arbitrary limits
  const paragraphs = Array.from(doc.querySelectorAll('p, li, blockquote, td'))
    .map(el => el.textContent.trim())
    .filter(text => text.length > 10);

  // 4. Extract all files and links (especially useful for Canvas Modules pages)
  const links = Array.from(doc.querySelectorAll('a[href]'))
    .map(a => {
      let href = a.href;
      try {
        // Resolve relative links if parsing an offscreen document
        href = new URL(a.getAttribute('href'), currentUrl).href;
      } catch (e) {}
      return {
        text: a.textContent.trim(),
        href: href
      };
    })
    .filter(link => link.text.length > 0 && !link.href.startsWith('javascript:'));

  // 5. Extract large continuous blocks of HTML (not the entire page)
  const html_blocks = [];
  const primaryNodes = doc.querySelectorAll('article, main, section, .user_content, #content, .module-sequence-footer-button--previous, .module-sequence-footer-button--next');
  
  if (primaryNodes.length > 0) {
    primaryNodes.forEach(node => html_blocks.push(node.outerHTML));
  } else {
    // Fallback: grab top level divs that have substantial text
    const divs = Array.from(doc.querySelectorAll('body > div, body > form'))
      .filter(div => (div.textContent || '').length > 500);
    divs.forEach(div => html_blocks.push(div.outerHTML));
  }

  return {
    url: currentUrl,
    title: doc.title,
    headers: headers,
    body: paragraphs,
    links: links,
    fullText: fullText,
    // Large continuous HTML blocks
    html_blocks: html_blocks
  };
}

// Add a floating button to trigger evaluation
function injectEvaluateButton() {
  const btn = document.createElement('button');
  btn.innerText = 'ADK: Eval Completeness';
  btn.style.position = 'fixed';
  btn.style.bottom = '20px';
  btn.style.right = '20px';
  btn.style.zIndex = '999999';
  btn.style.padding = '10px 15px';
  btn.style.backgroundColor = '#ff5500'; // Match app accent
  btn.style.color = '#ffffff';
  btn.style.border = 'none';
  btn.style.borderRadius = '4px';
  btn.style.fontFamily = 'monospace';
  btn.style.cursor = 'pointer';
  btn.style.boxShadow = '0 4px 6px rgba(0,0,0,0.1)';
  
  btn.onclick = () => {
    btn.innerText = 'Evaluating...';
    const payload = extractCourseData();
    
    chrome.runtime.sendMessage({
      action: 'evaluate_course_content',
      payload: payload
    }, (response) => {
      if (chrome.runtime.lastError) {
        console.error(chrome.runtime.lastError);
        btn.innerText = 'Error (Extension)';
        btn.style.backgroundColor = '#d9534f';
        return;
      }
      if (response && response.status === 'success') {
        btn.innerText = 'Sent to Node Stream';
        setTimeout(() => btn.innerText = 'ADK: Eval Completeness', 2000);
      } else {
        btn.innerText = 'Error (Port 3222 Down?)';
        btn.style.backgroundColor = '#d9534f';
      }
    });
  };
  
  document.body.appendChild(btn);
}

// Run injection
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', injectEvaluateButton);
} else {
  injectEvaluateButton();
}

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === 'start_nested_crawl') {
    performNestedCrawl(window.location.href);
    sendResponse({ status: 'started' });
  }
});

async function performNestedCrawl(startUrl) {
  console.log('Starting nested crawl from:', startUrl);
  const visited = new Set();
  const queue = [startUrl];
  const origin = new URL(startUrl).origin;
  
  while (queue.length > 0) {
    const currentUrl = queue.shift();
    if (visited.has(currentUrl)) continue;
    visited.add(currentUrl);
    
    console.log(`[Crawl] Fetching: ${currentUrl}`);
    try {
      const response = await fetch(currentUrl);
      const html = await response.text();
      const parser = new DOMParser();
      const doc = parser.parseFromString(html, 'text/html');
      
      const payload = extractCourseData(doc, currentUrl);
      
      // Send payload to background script to forward to adk-rust
      chrome.runtime.sendMessage({
        action: 'evaluate_course_content',
        payload: payload
      });
      
      // Extract same-origin links
      Array.from(doc.querySelectorAll('a[href]')).forEach(a => {
        try {
          const href = a.getAttribute('href');
          if (!href || href.startsWith('javascript:') || href.startsWith('#') || href.startsWith('mailto:')) return;
          
          const fullUrl = new URL(href, currentUrl).href.split('#')[0]; // Remove hash
          const linkOrigin = new URL(fullUrl).origin;
          
          if (linkOrigin === origin && !visited.has(fullUrl) && !queue.includes(fullUrl)) {
            queue.push(fullUrl);
          }
        } catch (e) {
          // ignore invalid urls
        }
      });
      
      // Wait 1 second before next fetch to avoid rate limits
      await new Promise(r => setTimeout(r, 1000));
      
    } catch (err) {
      console.error(`[Crawl] Failed to fetch ${currentUrl}:`, err);
    }
  }
  console.log('[Crawl] Finished crawling', visited.size, 'pages.');
}

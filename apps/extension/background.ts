// Nexum browser extension background service worker

type ServerConfig = {
  address: string;
};

type NexumTask = {
  id: string;
  source: string;
  destination: string;
  state: string;
  downloaded_bytes: number;
  total_bytes: number | null;
};

/** Add context menu item "Send to Nexum" for links */
chrome.contextMenus.create({
  id: "sendToNexum",
  title: "Send to Nexum",
  contexts: ["link"],
});

/** Listen for context menu clicks */
chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId === "sendToNexum" && info.linkUrl) {
    sendToNexum(info.linkUrl, tab?.id);
  }
});

/** Get the configured server address */
async function getServerAddress(): Promise<string> {
  const result = await chrome.storage.local.get(["server"]);
  const config = result as ServerConfig;
  return config.address || "127.0.0.1:39100";
}

/** Send a URL to the Nexum server */
async function sendToNexum(url: string, tabId?: number): Promise<void> {
  const server = await getServerAddress();
  try {
    const task = await createTask(server, url);
    // Show notification with task ID
    chrome.notifications?.create({
      type: "basic",
      iconUrl: "icons/nexum-48.png",
      title: "Nexum",
      message: `Task created: ${task.id}`,
      priority: 1,
    });
    // Notify content script if tabId provided
    if (tabId !== undefined) {
      chrome.tabs?.sendMessage(tabId, { type: "taskCreated", task });
    }
  } catch (error) {
    chrome.notifications?.create({
      type: "basic",
      iconUrl: "icons/nexum-48.png",
      title: "Nexum",
      message: `Failed: ${(error as Error).message}`,
      priority: 1,
    });
  }
}

/** Create a task on the Nexum server via JSON-RPC */
async function createTask(server: string, source: string): Promise<NexumTask> {
  const id = `browser-${Date.now()}`;
  const destination = `/tmp/nexum-${id}`;

  // Use fetch to call the server (JSON-RPC over HTTP)
  const response = await fetch(`http://${server}/jsonrpc`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "task.create",
      params: { id, source, destination },
    }),
  });

  const data = await response.json();
  if (data.error) {
    throw new Error(data.error.message || "Unknown error");
  }
  return data.result as NexumTask;
}

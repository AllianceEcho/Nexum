// Nexum browser extension popup

/** Save server address to storage */
function saveServer(): void {
  const server = (document.getElementById("server") as HTMLInputElement).value;
  chrome.storage.local.set({ server });
  (document.getElementById("status") as HTMLDivElement).textContent = "Server saved";
}

/** Load server address from storage and display tasks */
function loadTasks(): void {
  chrome.storage.local.get(["server"]).then((result: { server?: string }) => {
    const server = (document.getElementById("server") as HTMLInputElement);
    server.value = result.server || "127.0.0.1:39100";
  });
}

// Event listeners
document.getElementById("save")?.addEventListener("click", saveServer);
window.addEventListener("load", () => { loadTasks(); });

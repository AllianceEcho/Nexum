// Nexum browser extension content script

/** Inject "Send to Nexum" indicator on links when hovering */
document.addEventListener("DOMContentLoaded", () => {
  // Add hover listener to all links
  const links = document.querySelectorAll("a[href]");
  links.forEach((link) => {
    link.addEventListener("mouseenter", () => {
      // Show small Nexum badge when hovering links that could be downloads
      const href = (link as HTMLAnchorElement).getAttribute("href") || "";
      if (isDownloadUrl(href)) {
        showNexumBadge(link);
      }
    });
    link.addEventListener("mouseleave", () => {
      removeNexumBadge();
    });
  });
});

/** Send a URL to the background service worker */
function sendToNexum(url: string): void {
  chrome.runtime?.sendMessage({ type: "sendToNexum", url });
}

/** Check if URL looks like a downloadable resource */
function isDownloadUrl(url: string): boolean {
  const extensions = /\.(pdf|zip|tar|gz|mp4|mkv|mp3|exe|dmg|iso|torrent)$/i;
  return extensions.test(url) || /\.(pdf|zip|tar|gz|mp4|mkv|mp3|exe|dmg|iso)$/.test(url);
}

/** Show a small Nexum badge near a link */
function showNexumBadge(link: Element): void {
  removeNexumBadge();
  const badge = document.createElement("div");
  badge.id = "nexum-badge";
  badge.style.cssText = `
    position: fixed;
    right: 10px;
    top: ${link.getBoundingClientRect().top}px;
    background: #1a1a2e;
    color: white;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    z-index: 99999;
    cursor: pointer;
  `;
  badge.textContent = "Nexum";
  badge.onclick = () => sendToNexum((link as HTMLAnchorElement).getAttribute("href") || "");
  document.body.appendChild(badge);
}

/** Remove the Nexum badge if it exists */
function removeNexumBadge(): void {
  const badge = document.getElementById("nexum-badge");
  if (badge) badge.remove();
}

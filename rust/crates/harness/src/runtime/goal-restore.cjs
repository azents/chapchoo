// Relay only the current launch's already formatted native hook response.
const { createConnection } = require("node:net");
const path = process.env.CHAPCHOO_GOAL_RESTORE_SOCKET;
if (!path) return;
const peer = createConnection(path);
const chunks = [];
let size = 0;
const deadline = setTimeout(() => peer.destroy(), 1500);
peer.on("error", () => {});
peer.on("close", () => clearTimeout(deadline));
peer.on("data", (chunk) => {
  size += chunk.length;
  if (size > 20 * 1024) peer.destroy();
  else chunks.push(chunk);
});
peer.on("end", () => {
  if (!size || size > 20 * 1024) return;
  const response = Buffer.concat(chunks);
  try {
    JSON.parse(response);
    process.stdout.write(response);
  } catch {
    // Closed or partial responses carry no newly restored context.
  }
});

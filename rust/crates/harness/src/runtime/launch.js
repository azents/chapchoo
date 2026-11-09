#!/usr/bin/env node
// A live adapter gets one native launch grant. Its internal recovery cannot
// replace a possibly surviving predecessor; the Connector must retire the scope.
const { openSync, closeSync } = require("node:fs");
const { spawn } = require("node:child_process");
const executable = __CHAPCHOO_EXECUTABLE__;
const marker = __CHAPCHOO_MARKER__;
try {
  closeSync(openSync(marker, "wx", 0o600));
} catch {
  process.stderr.write("Native predecessor retirement is unconfirmed.\n");
  process.exitCode = 75;
  return;
}
const child = spawn(executable, process.argv.slice(2), {
  stdio: "inherit",
  env: { ...process.env, CODEX_PATH: executable },
});
for (const signal of ["SIGTERM", "SIGINT", "SIGHUP"]) {
  process.on(signal, () => child.kill(signal));
}
child.once("error", () => {
  process.stderr.write("Native executable launch failed.\n");
  process.exitCode = 74;
});
child.once("close", (code, signal) => {
  process.exitCode = code ?? (signal ? 128 : 74);
});

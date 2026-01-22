#!/usr/bin/env node
const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

const binName = process.platform === "win32" ? "agents.exe" : "agents";
const binPath = path.join(__dirname, binName);

if (!fs.existsSync(binPath)) {
  console.error("agents: binary not found. Reinstall the package to rebuild it.");
  process.exit(1);
}

const child = spawn(binPath, process.argv.slice(2), { stdio: "inherit" });

child.on("error", (err) => {
  console.error("agents: failed to start", err.message);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.exit(1);
  }
  process.exit(code ?? 1);
});

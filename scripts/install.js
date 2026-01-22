#!/usr/bin/env node
const { execFileSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const targetDir = path.join(root, "target", "release");
const binDir = path.join(root, "bin");
const binName = process.platform === "win32" ? "agents.exe" : "agents";
const builtPath = path.join(targetDir, binName);
const outPath = path.join(binDir, binName);

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function runCargoBuild() {
  try {
    execFileSync("cargo", ["build", "-p", "agents-cli", "--release"], {
      stdio: "inherit",
      cwd: root,
    });
  } catch (err) {
    console.error("agents: failed to build via cargo");
    console.error("agents: ensure Rust is installed (https://rustup.rs)");
    process.exit(1);
  }
}

function installBinary() {
  if (!fs.existsSync(builtPath)) {
    console.error("agents: build output missing at", builtPath);
    process.exit(1);
  }

  ensureDir(binDir);
  fs.copyFileSync(builtPath, outPath);

  if (process.platform !== "win32") {
    fs.chmodSync(outPath, 0o755);
  }
}

runCargoBuild();
installBinary();

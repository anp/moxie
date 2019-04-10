#!/usr/bin/env node

const { spawnSync } = require("child_process");
const fs = require("fs");

function run(cmd, args, opts) {
  const output = spawnSync(cmd, args, opts);

  if (output.error != null) {
    throw output.error;
  }

  if (output.status !== 0) {
    throw new Error("Bad error code when running `" + cmd + " " + args.join(" ") + "`: " + output.status);
  }
}

let folderName = '.';

if (process.argv.length >= 3) {
  folderName = process.argv[2];
  if (!fs.existsSync(folderName)) {
    fs.mkdirSync(folderName);
  }
}

run("git", ["clone", "https://github.com/rustwasm/rust-webpack-template.git", folderName]);

console.log(" 🦀 Rust + 🕸 WebAssembly + Webpack = ❤️ ");

run("npm", ["install"], { cwd: folderName, shell: true });

console.log(" Installed dependencies ✅ ");

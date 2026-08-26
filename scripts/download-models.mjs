// Download FunASR GGUF models via Node's TLS stack (bypasses schannel credential issues).
// Supports HTTP redirects and resume of partially downloaded files. Usage:
//   node scripts/download-models.mjs [baseUrl]
// Default baseUrl is https://hf-mirror.com (override with e.g. https://huggingface.co).

import { createWriteStream, existsSync, statSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import https from "node:https";

const __dirname = dirname(fileURLToPath(import.meta.url));
const outDir = join(__dirname, "..", "src-tauri", "resources", "models");
const base = process.argv[2] ?? "https://hf-mirror.com";

const files = [
  `${base}/FunAudioLLM/SenseVoiceSmall-GGUF/resolve/main/sensevoice-small-q8.gguf`,
  `${base}/FunAudioLLM/fsmn-vad-GGUF/resolve/main/fsmn-vad.gguf`,
];

function get(url, headers) {
  return new Promise((resolve, reject) => {
    const req = https.get(url, { headers }, (res) => resolve(res));
    req.on("error", reject);
    req.setTimeout(30000, () => req.destroy(new Error("connect timeout")));
  });
}

async function follow(url, headers, depth = 0) {
  if (depth > 8) throw new Error("too many redirects");
  const res = await get(url, headers);
  if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
    res.resume();
    return follow(new URL(res.headers.location, url).toString(), headers, depth + 1);
  }
  return res;
}

async function download(url) {
  const name = decodeURIComponent(new URL(url).pathname.split("/").pop());
  const dest = join(outDir, name);
  mkdirSync(outDir, { recursive: true });
  const have = existsSync(dest) ? statSync(dest).size : 0;

  const headRes = await follow(url, { Range: `bytes=${have}-` });
  const total = Number(headRes.headers["content-length"] ?? 0) + (headRes.statusCode === 206 ? have : 0);
  if (headRes.statusCode === 200 && have > 0) {
    // server ignored Range; restart
  }
  const offset = headRes.statusCode === 206 ? have : 0;
  if (total > 0 && total === offset) {
    console.log(`${name}: complete (${total} bytes), skip`);
    headRes.resume();
    return;
  }
  console.log(`${name}: ${offset > 0 ? `resume at ${offset}` : "download"}${total ? ` / ${total}` : ""}`);
  const file = createWriteStream(dest, { flags: offset > 0 ? "a" : "w" });
  let lastPct = -10;
  await new Promise((resolve, reject) => {
    let received = offset;
    headRes.on("data", (chunk) => {
      received += chunk.length;
      const pct = total ? Math.floor((received / total) * 100) : 0;
      if (pct >= lastPct + 10) {
        lastPct = pct;
        console.log(`${name}: ${pct}% (${received} bytes)`);
      }
    });
    headRes.pipe(file);
    headRes.on("error", reject);
    file.on("finish", resolve);
    file.on("error", reject);
  });
  console.log(`${name}: done (${statSync(dest).size} bytes)`);
}

for (const url of files) {
  let lastErr;
  for (let attempt = 1; attempt <= 3; attempt++) {
    try {
      await download(url);
      lastErr = undefined;
      break;
    } catch (err) {
      lastErr = err;
      console.error(`attempt ${attempt} failed: ${err.message}`);
      await new Promise((r) => setTimeout(r, 2000));
    }
  }
  if (lastErr) {
    console.error(`FAILED: ${url}`);
    process.exitCode = 1;
  }
}

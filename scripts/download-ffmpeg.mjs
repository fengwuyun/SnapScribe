// Download a Windows x64 ffmpeg essentials build and extract ffmpeg.exe/ffprobe.exe
// into src-tauri/resources/bin/. Usage: node scripts/download-ffmpeg.mjs [zipUrl]
import { createWriteStream, existsSync, statSync, mkdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import https from "node:https";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const url =
  process.argv[2] ??
  "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";
const zipPath = join(root, ".ffmpeg.zip");
const outBin = join(root, "src-tauri", "resources", "bin");

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
  if (res.statusCode !== 200) throw new Error(`HTTP ${res.statusCode}`);
  return res;
}

console.log(`downloading ${url}`);
const res = await follow(url);
const total = Number(res.headers["content-length"] ?? 0);
await new Promise((resolve, reject) => {
  let received = 0;
  let lastPct = -10;
  res.on("data", (chunk) => {
    received += chunk.length;
    const pct = total ? Math.floor((received / total) * 100) : 0;
    if (pct >= lastPct + 10) { lastPct = pct; console.log(`${pct}% (${received})`); }
  });
  res.pipe(createWriteStream(zipPath));
  res.on("error", reject);
  res.on("end", resolve);
});
console.log(`zip: ${statSync(zipPath).size} bytes`);

// Extract only the two executables we ship.
execFileSync("powershell", [
  "-NoProfile", "-Command",
  `$tmp = Join-Path $env:TEMP "snapscribe-ff";` +
  ` if (Test-Path $tmp) { Remove-Item $tmp -Recurse -Force };` +
  ` Expand-Archive -LiteralPath "${zipPath}" -DestinationPath $tmp -Force;` +
  ` New-Item -ItemType Directory -Force -Path "${outBin}" | Out-Null;` +
  ` Copy-Item (Get-ChildItem $tmp -Recurse -Filter ffmpeg.exe | Select-Object -First 1).FullName "${join(outBin, "ffmpeg.exe")}";` +
  ` Copy-Item (Get-ChildItem $tmp -Recurse -Filter ffprobe.exe | Select-Object -First 1).FullName "${join(outBin, "ffprobe.exe")}"`,
]);
rmSync(zipPath, { force: true });
for (const name of ["ffmpeg.exe", "ffprobe.exe"]) {
  const p = join(outBin, name);
  console.log(name, existsSync(p) ? `${statSync(p).size} bytes OK` : "MISSING");
}

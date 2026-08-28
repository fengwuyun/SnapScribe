import { createHash } from "node:crypto";
import {
  copyFileSync,
  createWriteStream,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
} from "node:fs";
import https from "node:https";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const output = join(root, "src-tauri", "resources", "bin", "llama-funasr-sensevoice.exe");
const url = "https://github.com/QwenAudio/SenseVoice/releases/download/runtime-llamacpp-v0.1.9/funasr-llamacpp-windows-x64.zip";
const expectedSha256 = "6767af74e42c8b928742e12d5995c139636d9482ea151cdbb51f1b7573667772";

function get(source, depth = 0) {
  if (depth > 8) throw new Error("too many redirects");
  return new Promise((resolveRequest, reject) => {
    const request = https.get(source, (response) => {
      if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
        response.resume();
        resolveRequest(get(new URL(response.headers.location, source).toString(), depth + 1));
      } else if (response.statusCode === 200) {
        resolveRequest(response);
      } else {
        response.resume();
        reject(new Error(`HTTP ${response.statusCode}`));
      }
    });
    request.on("error", reject);
    request.setTimeout(30000, () => request.destroy(new Error("connect timeout")));
  });
}

function findFile(directory, name) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      const nested = findFile(path, name);
      if (nested) return nested;
    } else if (entry.name.toLowerCase() === name.toLowerCase()) {
      return path;
    }
  }
  return null;
}

const tempRoot = mkdtempSync(join(tmpdir(), "snapscribe-asr-"));
const archive = join(tempRoot, "runtime.zip");
const extracted = join(tempRoot, "extracted");
try {
  console.log(`downloading ${url}`);
  const response = await get(url);
  await new Promise((resolveDownload, reject) => {
    const file = createWriteStream(archive);
    response.pipe(file);
    response.on("error", reject);
    file.on("finish", resolveDownload);
    file.on("error", reject);
  });
  const digest = createHash("sha256").update(readFileSync(archive)).digest("hex");
  if (digest !== expectedSha256) throw new Error(`SHA-256 mismatch: ${digest}`);
  mkdirSync(extracted);
  execFileSync("tar.exe", ["-xf", archive, "-C", extracted]);
  const executable = findFile(extracted, "llama-funasr-sensevoice.exe");
  if (!executable) throw new Error("archive does not contain llama-funasr-sensevoice.exe");
  mkdirSync(dirname(output), { recursive: true });
  copyFileSync(executable, output);
  if (!existsSync(output)) throw new Error("runtime copy failed");
  console.log(`runtime ready: ${output}`);
} finally {
  const resolvedTemp = resolve(tempRoot);
  if (!resolvedTemp.startsWith(resolve(tmpdir()))) throw new Error("refusing to remove non-temporary path");
  rmSync(resolvedTemp, { recursive: true, force: true });
}

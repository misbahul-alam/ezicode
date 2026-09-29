#!/usr/bin/env node
/**
 * ezi — zero-install launcher for the ezicode native editor.
 *
 * Usage:
 *   npx ezi [file|dir] [args...]   open editor (downloads binary on first run)
 *   npx ezi --help                  show help
 *   npx ezi --version               launcher + editor version
 *
 * Env overrides:
 *   EZI_VERSION   GitHub tag version without `v` (default: package version)
 *   EZI_REPO      `owner/repo` (default: olovalabs/ezicode)
 *   EZI_CACHE     cache dir override
 *
 * Design: zero npm dependencies (node builtins only) so the published
 * package stays ~5KB. Platform binaries come from GitHub Releases:
 *   linux x64   -> ezicode-x86_64-unknown-linux-gnu.tar.gz
 *   mac arm64   -> ezicode-aarch64-apple-darwin.tar.gz
 *   win x64     -> ezicode-x86_64-pc-windows-msvc.exe (standalone, no unzip)
 */

import * as fs from "node:fs";
import * as https from "node:https";
import * as os from "node:os";
import * as path from "node:path";
import { spawn, execFile } from "node:child_process";

const LAUNCHER_VERSION = "2.0.1"; // npm package version
const EDITOR_VERSION = "0.1.4"; // keep in sync with app/Cargo.toml (default download)
const REPO = process.env.EZI_REPO || "olovalabs/ezicode";
const TAG = `v${process.env.EZI_VERSION || EDITOR_VERSION}`;

type Target =
  | { kind: "targz"; asset: string; binRel: string }
  | { kind: "exe"; asset: string };

function resolveTarget(): Target {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "linux" && arch === "x64") {
    const t = "x86_64-unknown-linux-gnu";
    return { kind: "targz", asset: `ezicode-${t}.tar.gz`, binRel: `ezicode-${t}/ezicode` };
  }
  if (platform === "darwin" && arch === "arm64") {
    const t = "aarch64-apple-darwin";
    return { kind: "targz", asset: `ezicode-${t}.tar.gz`, binRel: `ezicode-${t}/ezicode` };
  }
  if (platform === "win32" && arch === "x64") {
    // Standalone exe on the release — no zip extraction needed.
    return { kind: "exe", asset: "ezicode-x86_64-pc-windows-msvc.exe" };
  }
  throw new Error(
    `ezi: unsupported platform ${platform}/${arch}. ` +
      `Prebuilt binaries exist for linux-x64, darwin-arm64, win32-x64 only. ` +
      `Build from source: https://github.com/${REPO}`
  );
}

function cacheDir(): string {
  if (process.env.EZI_CACHE) return process.env.EZI_CACHE;
  if (process.platform === "win32") {
    const base = process.env.LOCALAPPDATA || path.join(os.homedir(), "AppData", "Local");
    return path.join(base, "ezi", "cache", TAG);
  }
  const xdg = process.env.XDG_CACHE_HOME || path.join(os.homedir(), ".cache");
  return path.join(xdg, "ezi", TAG);
}

function download(url: string, dest: string, redirects = 5): Promise<void> {
  return new Promise((resolve, reject) => {
    const req = https.get(url, { headers: { "User-Agent": "ezi-launcher" } }, (res) => {
      const status = res.statusCode || 0;
      if (status >= 300 && status < 400 && res.headers.location) {
        if (redirects === 0) {
          reject(new Error(`ezi: too many redirects downloading ${url}`));
          return;
        }
        res.resume();
        download(res.headers.location, dest, redirects - 1).then(resolve, reject);
        return;
      }
      if (status !== 200) {
        res.resume();
        reject(new Error(`ezi: download failed (${status}) for ${url}`));
        return;
      }
      const out = fs.createWriteStream(dest);
      res.pipe(out);
      out.on("finish", () => out.close(() => resolve()));
      out.on("error", (err) => {
        fs.rmSync(dest, { force: true });
        reject(err);
      });
    });
    req.on("error", reject);
  });
}

function extractTargz(archive: string, destDir: string): Promise<void> {
  return new Promise((resolve, reject) => {
    // System tar exists on linux + macOS runners and images. Zero-dep by design.
    execFile("tar", ["-xzf", archive, "-C", destDir], (err, _stdout, stderr) => {
      if (err) {
        reject(new Error(`ezi: failed to extract ${archive}: ${stderr || err.message}`));
        return;
      }
      resolve();
    });
  });
}

async function ensureBinary(verbose: boolean): Promise<string> {
  const target = resolveTarget();
  const dir = cacheDir();
  fs.mkdirSync(dir, { recursive: true });

  if (target.kind === "exe") {
    const bin = path.join(dir, "ezicode.exe");
    if (fs.existsSync(bin)) return bin;
    const url = `https://github.com/${REPO}/releases/download/${TAG}/${target.asset}`;
    const tmp = bin + ".download";
    if (verbose) console.log(`ezi: downloading ${url}`);
    await download(url, tmp);
    fs.renameSync(tmp, bin);
    return bin;
  }

  const bin = path.join(dir, target.binRel);
  if (fs.existsSync(bin)) {
    fs.chmodSync(bin, 0o755);
    return bin;
  }
  const url = `https://github.com/${REPO}/releases/download/${TAG}/${target.asset}`;
  const archive = path.join(dir, target.asset);
  if (!fs.existsSync(archive)) {
    if (verbose) console.log(`ezi: downloading ${url}`);
    await download(url, archive);
  }
  if (verbose) console.log(`ezi: extracting ${target.asset}`);
  await extractTargz(archive, dir);
  if (!fs.existsSync(bin)) throw new Error(`ezi: binary not found after extract: ${bin}`);
  fs.chmodSync(bin, 0o755);
  return bin;
}

function printHelp(): void {
  console.log(`ezi ${LAUNCHER_VERSION} — launcher for ezicode (https://github.com/${REPO})

Usage:
  ezi [file|dir] [editor-args...]   open in ezicode (downloads binary first run)
  ezi --help                        this help
  ezi --version                     launcher version
  ezi --where                       print cached binary path
  ezi --clean                       remove cached binaries

Env:
  EZI_VERSION   release version (default ${EDITOR_VERSION})
  EZI_REPO      owner/repo (default ${REPO})
  EZI_CACHE     cache dir override
`);
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const verbose = args.includes("--verbose") || !!process.env.EZI_VERBOSE;
  const passthrough = args.filter((a) => a !== "--verbose");

  if (passthrough.includes("--help") || passthrough.includes("-h")) {
    printHelp();
    return;
  }
  if (passthrough.includes("--version") || passthrough.includes("-V")) {
    console.log(`ezi ${LAUNCHER_VERSION} (editor ${TAG} @ ${REPO})`);
    return;
  }
  if (passthrough.includes("--clean")) {
    fs.rmSync(cacheDir(), { recursive: true, force: true });
    console.log("ezi: cache cleared");
    return;
  }

  const bin = await ensureBinary(verbose);

  if (passthrough.includes("--where")) {
    console.log(bin);
    return;
  }

  // Launch detached so `npx ezi` returns immediately while the GUI stays open.
  // stdio ignored: the editor owns its own window, not the terminal.
  const child = spawn(bin, passthrough, {
    detached: true,
    stdio: "ignore",
    windowsHide: true,
  });
  child.unref();
  if (verbose) console.log(`ezi: launched ${bin}`);
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : String(err));
  process.exit(1);
});

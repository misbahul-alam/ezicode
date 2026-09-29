# ezi — `npx ezi` launcher for ezicode

Zero-dependency TypeScript CLI. It detects OS/arch, downloads the matching
native `ezicode` binary from GitHub Releases on first run, caches it, and
launches the editor detached. No `npm install` required by users.

## Mono-repo layout

```
ezicode/            # Rust workspace (cargo members: app)
├── app/            # native editor (ezicode binary)
├── ezi-cli/        # npm launcher (this folder, NOT a cargo member)
```

Cargo ignores `ezi-cli/` (`members = ["app"]` in root `Cargo.toml:2`), npm
ignores the Rust side. One repo, two package managers.

## Develop

```bash
cd ezi-cli
npm install
npm run build     # tsc -> dist/index.js
node ./dist/index.js --help
EZI_VERBOSE=1 node ./dist/index.js .   # downloads real binary, opens editor
```

## Publish

```bash
cd ezi-cli
npm run build
npm publish       # package `ezi`, bin `ezi` -> dist/index.js
```

Users then run:

```bash
npx -y ezi@latest .  # first run downloads ~30-60MB binary to cache, opens editor
ezi .             # after npm i -g ezi
```

Version resolution (no republish needed for editor releases):

1. `EZI_VERSION=0.1.2` pins that release, no network.
2. Unset (or `EZI_VERSION=latest`, just `npx ezi`) checks
   `api.github.com/repos/<owner>/<repo>/releases/latest` on every run.
   Same tag reuses the cached binary (no download); new tag downloads once
   to `<cache>/vX.Y.Z/`. Offline / API failure reuses the last-seen tag,
   else the compiled-in default near `app/Cargo.toml`.

Only republish `ezi` for launcher bugfixes. Skip the lookup with
`EZI_NO_UPDATE_CHECK=1`. Optional `EZI_GITHUB_TOKEN` raises API rate limits.

## Cache

- Linux/macOS: `~/.cache/ezi/v<version>/` (or `$XDG_CACHE_HOME`) + `latest.json` tag cache
- Windows: `%LOCALAPPDATA%/ezi/cache/v<version>/`
- Override: `EZI_CACHE`, `EZI_VERSION`, `EZI_REPO`, `EZI_NO_UPDATE_CHECK=1`

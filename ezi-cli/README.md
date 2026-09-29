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
npx -y ezi .      # first run downloads ~30-60MB binary to cache, opens editor
ezi .             # after npm i -g ezi
```

Keep `VERSION` in `src/index.ts`, `version` in `package.json`, and `version`
in `app/Cargo.toml` in sync — the launcher builds its download URL as
`.../releases/download/v<VERSION>/ezicode-<target>.*`.

## Cache

- Linux/macOS: `~/.cache/ezi/v<version>/` (or `$XDG_CACHE_HOME`)
- Windows: `%LOCALAPPDATA%/ezi/cache/v<version>/`
- Override: `EZI_CACHE`, `EZI_VERSION`, `EZI_REPO`

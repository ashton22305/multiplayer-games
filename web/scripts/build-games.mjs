// Builds each Rust game crate to wasm and emits a per-game iframe bundle under
// <out>/<name>/ (index.html + <name>.wasm) plus <out>/_shared/ (loader + host bridge).
// Default out dir is web/public/games so local dev works unchanged.
//
// Usage:
//   node scripts/build-games.mjs                      # build all crates under crates/games
//   node scripts/build-games.mjs snake                # build only the named game(s)
//   node scripts/build-games.mjs --out games-dist     # write to a custom dir (standalone deploy)
//   node scripts/build-games.mjs snake --out out/     # combine name filter with custom out dir
import { execFileSync } from 'node:child_process'
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  statSync,
  writeFileSync,
} from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const webDir = resolve(here, '..')
const repoRoot = resolve(webDir, '..')
const gamesSrcDir = join(repoRoot, 'crates', 'games')
const sharedSrc = join(webDir, 'public', 'games', '_shared')
const target = 'wasm32-unknown-unknown'
const releaseDir = join(repoRoot, 'target', target, 'release')

// Parse args: --out <dir> extracts the output dir; remaining args are game names.
const args = process.argv.slice(2)
let outBase = join(webDir, 'public', 'games')
const gameArgs = []
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--out') {
    if (!args[i + 1]) { console.error('build-games: --out requires a path'); process.exit(1) }
    outBase = resolve(args[++i])
  } else {
    gameArgs.push(args[i])
  }
}

function discoverGames() {
  return readdirSync(gamesSrcDir, { withFileTypes: true })
    .filter((d) => d.isDirectory() && existsSync(join(gamesSrcDir, d.name, 'Cargo.toml')))
    .map((d) => d.name)
}

function hasWasmOpt() {
  try {
    execFileSync('wasm-opt', ['--version'], { stdio: 'ignore' })
    return true
  } catch {
    return false
  }
}

function title(name) {
  return name.charAt(0).toUpperCase() + name.slice(1)
}

function indexHtml(name) {
  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1, user-scalable=no" />
<title>${title(name)}</title>
<style>
  html, body { margin: 0; padding: 0; width: 100%; height: 100%; overflow: hidden; background: #000; }
  #glcanvas { display: block; width: 100%; height: 100%; outline: none; }
</style>
</head>
<body>
<canvas id="glcanvas" tabindex="0"></canvas>
<!-- The bundled quad_net plugin assigns register_plugin without declaring it,
     which throws under the bundle's strict mode. Pre-declare the global. -->
<script>var register_plugin;</script>
<script src="../_shared/mq_js_bundle.js"></script>
<script src="../_shared/host.js" data-wasm="${name}.wasm"></script>
</body>
</html>
`
}

const games = gameArgs.length ? gameArgs : discoverGames()
if (games.length === 0) {
  console.log('build-games: no game crates found under crates/games')
  process.exit(0)
}

console.log(`build-games: ${games.join(', ')}`)
execFileSync(
  'cargo',
  ['build', '--release', '--target', target, ...games.flatMap((g) => ['-p', g])],
  { cwd: repoRoot, stdio: 'inherit' },
)

const optimize = hasWasmOpt()
if (!optimize) console.log('build-games: wasm-opt not found, shipping unoptimized wasm')

for (const name of games) {
  const wasmIn = join(releaseDir, `${name}.wasm`)
  if (!existsSync(wasmIn)) throw new Error(`build-games: expected ${wasmIn} after cargo build`)

  const outDir = join(outBase, name)
  mkdirSync(outDir, { recursive: true })
  const wasmOut = join(outDir, `${name}.wasm`)

  if (optimize) {
    execFileSync('wasm-opt', ['-Oz', wasmIn, '-o', wasmOut], { stdio: 'inherit' })
  } else {
    copyFileSync(wasmIn, wasmOut)
  }
  writeFileSync(join(outDir, 'index.html'), indexHtml(name))

  const kb = (statSync(wasmOut).size / 1024).toFixed(0)
  console.log(`  ${name}: ${kb} KB -> ${outDir}`)
}

// Always sync _shared into the output tree so the bundle is self-contained when
// --out points somewhere other than web/public/games (games-only deploy).
const sharedDst = join(outBase, '_shared')
if (sharedSrc !== sharedDst) {
  mkdirSync(sharedDst, { recursive: true })
  for (const file of readdirSync(sharedSrc)) {
    copyFileSync(join(sharedSrc, file), join(sharedDst, file))
    console.log(`  _shared/${file} -> ${sharedDst}`)
  }
}

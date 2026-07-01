// Builds each Rust game crate to wasm and copies the binary under <out>/<name>/.
// Default out dir is web/public/games so local dev works unchanged.
//
// Usage:
//   node scripts/build-games.mjs                      # build all crates under crates/games
//   node scripts/build-games.mjs snake                # build only the named game(s)
//   node scripts/build-games.mjs --out games-dist     # write to a custom dir
import { execFile, execFileSync } from 'node:child_process'
import { copyFileSync, existsSync, mkdirSync, readdirSync, statSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

const here = dirname(fileURLToPath(import.meta.url))
const webDir = resolve(here, '..')
const repoRoot = resolve(webDir, '..')
const gamesSrcDir = join(repoRoot, 'crates', 'games')
const target = 'wasm32-unknown-unknown'
const releaseDir = join(repoRoot, 'target', target, 'release')

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

await Promise.all(
  games.map(async (name) => {
    const wasmIn = join(releaseDir, `${name}.wasm`)
    if (!existsSync(wasmIn)) throw new Error(`build-games: expected ${wasmIn} after cargo build`)

    const outDir = join(outBase, name)
    mkdirSync(outDir, { recursive: true })
    const wasmOut = join(outDir, `${name}.wasm`)

    if (optimize) {
      await execFileAsync('wasm-opt', ['-Oz', wasmIn, '-o', wasmOut])
    } else {
      copyFileSync(wasmIn, wasmOut)
    }

    const kb = (statSync(wasmOut).size / 1024).toFixed(0)
    console.log(`  ${name}: ${kb} KB -> ${outDir}`)
  }),
)

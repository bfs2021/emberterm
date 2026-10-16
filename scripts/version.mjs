// One version for the whole repo: packages/desktop/Cargo.toml is the source (tauri.conf.json has no
// "version", so Tauri reads it from there); the package.json files must match it.
//   node scripts/version.mjs check [vX.Y.Z]   fail if any file disagrees (or with the given tag)
//   node scripts/version.mjs set X.Y.Z         write the version everywhere and refresh Cargo.lock
import { readFileSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";

const CARGO = "packages/desktop/Cargo.toml";
const PACKAGES = ["package.json", "packages/app/package.json", "packages/ui/package.json"];
const SEMVER = /^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/;

const cargoVersion = () => readFileSync(CARGO, "utf8").match(/^version = "([^"]+)"/m)?.[1];
const pkgVersion = (p) => JSON.parse(readFileSync(p, "utf8")).version;

function check(tag) {
  const found = { [CARGO]: cargoVersion(), ...Object.fromEntries(PACKAGES.map((p) => [p, pkgVersion(p)])) };
  const want = tag ? tag.replace(/^v/, "") : found[CARGO];
  const bad = Object.entries(found).filter(([, v]) => v !== want);
  if (bad.length) {
    console.error(`Expected version ${want}, found:`);
    for (const [file, v] of bad) console.error(`  ${file}: ${v ?? "(none)"}`);
    process.exit(1);
  }
  console.log(`All versions are ${want}`);
}

function set(version) {
  if (!SEMVER.test(version ?? "")) { console.error(`Not a version: ${version}`); process.exit(1); }
  const cargo = readFileSync(CARGO, "utf8");
  writeFileSync(CARGO, cargo.replace(/^version = "[^"]+"/m, `version = "${version}"`));
  for (const p of PACKAGES) {
    const raw = readFileSync(p, "utf8");
    writeFileSync(p, raw.replace(/^(\s*"version":\s*)"[^"]+"/m, `$1"${version}"`));
  }
  execFileSync("cargo", ["update", "--manifest-path", CARGO, "--workspace", "--offline"], { stdio: "inherit" });
  check();
}

const [cmd, arg] = process.argv.slice(2);
if (cmd === "check") check(arg);
else if (cmd === "set") set(arg);
else { console.error("usage: version.mjs check [vX.Y.Z] | set X.Y.Z"); process.exit(1); }

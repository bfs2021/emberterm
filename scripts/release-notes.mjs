// The GitHub release body is the tag's section of CHANGELOG.md.
//   node scripts/release-notes.mjs vX.Y.Z   print the notes; fail if a stable tag has no section
// A pre-release tag (vX.Y.Z-rc.1) without its own section gets a short placeholder, so release
// workflow tests do not need a changelog entry.
import { readFileSync } from "node:fs";

function releaseNotes(changelog, tag) {
  const version = tag.replace(/^v/, "");
  const lines = changelog.replace(/\r\n/g, "\n").split("\n");
  const start = lines.findIndex((l) => l.startsWith(`## [${version}]`));
  if (start >= 0) {
    const end = lines.findIndex((l, i) => i > start && l.startsWith("## "));
    const body = lines.slice(start + 1, end < 0 ? undefined : end).join("\n").trim();
    if (body) return body;
  }
  if (version.includes("-")) return `Pre-release ${tag} for testing the release workflow.`;
  throw new Error(`CHANGELOG.md has no entry for ${version}`);
}

const tag = process.argv[2];
if (!tag) {
  console.error("usage: node scripts/release-notes.mjs vX.Y.Z");
  process.exit(1);
}
try {
  console.log(releaseNotes(readFileSync("CHANGELOG.md", "utf8"), tag));
} catch (e) {
  console.error(e.message);
  process.exit(1);
}

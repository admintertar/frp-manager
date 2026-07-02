import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const workflow = readFileSync(
  join(root, ".github/workflows/release.yml"),
  "utf8",
);
const tauriConfig = JSON.parse(
  readFileSync(join(root, "src-tauri/tauri.conf.json"), "utf8"),
);

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

for (const label of ["macos-arm64", "macos-x64", "windows-x64", "linux-x64"]) {
  assert(workflow.includes(`label: ${label}`), `missing release matrix ${label}`);
}

assert(workflow.includes("workflow_dispatch:"), "release workflow must support manual runs");
assert(workflow.includes('"v*"'), "release workflow must run for v* tags");
assert(workflow.includes("tauri-apps/tauri-action@v1"), "workflow must use tauri-action");
assert(workflow.includes("pnpm/action-setup@v6"), "workflow must use pnpm/action-setup v6");
assert(!workflow.includes("pnpm/action-setup@v4"), "workflow must not use deprecated pnpm/action-setup v4");
assert(workflow.includes("pnpm install --frozen-lockfile"), "workflow must use frozen pnpm lockfile");
assert(workflow.includes("pnpm typecheck"), "workflow must run TypeScript checks");
assert(workflow.includes("pnpm build"), "workflow must build the frontend");
assert(workflow.includes("cargo test --manifest-path src-tauri/Cargo.toml"), "workflow must run Rust tests");
assert(workflow.includes("libwebkit2gtk-4.1-dev"), "Linux builds need WebKitGTK 4.1");
assert(
  workflow.includes("libayatana-appindicator3-dev"),
  "Linux tray builds need appindicator development libraries",
);
assert(
  workflow.includes("APPLE_SIGNING_IDENTITY"),
  "macOS CI builds should be ad-hoc signed when no certificate is configured",
);
assert(
  tauriConfig.bundle.icon.includes("icons/icon.icns") &&
    tauriConfig.bundle.icon.includes("icons/icon.ico"),
  "release builds should keep the current platform icons",
);

console.log("release workflow configuration ok");

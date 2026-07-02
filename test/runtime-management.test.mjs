import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("runtime management commands are exposed to the frontend", async () => {
  const lib = await readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const api = await readFile(new URL("../src/lib/api.ts", import.meta.url), "utf8");

  assert.match(lib, /commands::get_runtime_status/);
  assert.match(lib, /commands::install_runtime/);
  assert.match(api, /invokeOrFallback\("get_runtime_status"/);
  assert.match(api, /invokeOrFallback\("install_runtime"/);
});

test("runtime settings exposes install and update management states", async () => {
  const settings = await readFile(
    new URL("../src/components/RuntimeSettings.tsx", import.meta.url),
    "utf8",
  );
  const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
  const workbench = await readFile(
    new URL("../src/components/ProfileWorkbench.tsx", import.meta.url),
    "utf8",
  );

  assert.match(settings, /installRuntime/);
  assert.match(settings, /not installed/);
  assert.match(settings, /Download/);
  assert.match(settings, /Open Folder/);
  assert.match(app, /getRuntimeStatus/);
  assert.match(workbench, /runtimeInstalled/);
});

test("runtime settings actions wrap inside the modal", async () => {
  const css = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");
  const actionsBlock = css.match(
    /\.runtime-modal \.modal-actions\s*\{(?<body>[\s\S]*?)\}/,
  )?.groups?.body;
  const buttonBlock = css.match(
    /\.runtime-modal \.modal-actions \.command-button\s*\{(?<body>[\s\S]*?)\}/,
  )?.groups?.body;

  assert.ok(actionsBlock, "expected runtime modal action CSS block");
  assert.ok(buttonBlock, "expected runtime modal action button CSS block");
  assert.match(actionsBlock, /flex-wrap:\s*wrap/);
  assert.match(actionsBlock, /overflow:\s*hidden/);
  assert.match(buttonBlock, /min-width:\s*0/);
  assert.match(buttonBlock, /white-space:\s*nowrap/);
});

test("frpc is not bundled as a Tauri sidecar", async () => {
  const config = await readFile(
    new URL("../src-tauri/tauri.conf.json", import.meta.url),
    "utf8",
  );

  assert.doesNotMatch(config, /externalBin/);
  assert.doesNotMatch(config, /binaries\/frpc/);
});

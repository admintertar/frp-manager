import assert from "node:assert/strict";
import { access, readFile } from "node:fs/promises";
import { test } from "node:test";

test("app icon assets are generated from the bundled svg source", async () => {
  const svg = await readFile(
    new URL("../src-tauri/icons/icon.svg", import.meta.url),
    "utf8",
  );
  const config = JSON.parse(
    await readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
  );

  assert.match(svg, /viewBox="0 0 196 196"/);
  assert.match(svg, /#307fec/);
  assert.match(svg, /scale\(0\.84\)/);

  for (const iconPath of config.bundle.icon) {
    await access(new URL(`../src-tauri/${iconPath}`, import.meta.url));
  }
});

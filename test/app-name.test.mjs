import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("desktop bundle uses the FRP Manager app name", async () => {
  const config = JSON.parse(
    await readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
  );
  const pkg = JSON.parse(
    await readFile(new URL("../package.json", import.meta.url), "utf8"),
  );

  assert.equal(config.productName, "FRP Manager");
  assert.equal(config.app.windows[0].title, "FRP Manager");
  assert.equal(pkg.name, "frp-manager");
});

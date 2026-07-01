import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

const modalFiles = [
  "src/components/ProfileEditor.tsx",
  "src/components/ProxyEditor.tsx",
  "src/components/RuntimeSettings.tsx",
];

test("modal focus effects are not retriggered by onClose callback identity changes", async () => {
  for (const file of modalFiles) {
    const source = await readFile(new URL(`../${file}`, import.meta.url), "utf8");

    assert.doesNotMatch(
      source,
      /\}, \[onClose, open\]\);/,
      `${file} should keep initial focus tied to open state only`,
    );
  }
});

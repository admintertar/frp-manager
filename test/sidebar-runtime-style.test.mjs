import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

const css = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");

function cssBlock(selector) {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return css.match(new RegExp(`${escaped}\\s*\\{(?<body>[\\s\\S]*?)\\}`))?.groups
    ?.body;
}

function lastCssBlock(selector) {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return [...css.matchAll(new RegExp(`${escaped}\\s*\\{(?<body>[\\s\\S]*?)\\}`, "g"))]
    .at(-1)
    ?.groups?.body;
}

test("sidebar runtime footer keeps settings centered beside compact copy", () => {
  const runtimeBlock = cssBlock(".sidebar-runtime");
  const textBlock = cssBlock(".sidebar-runtime > div");
  const buttonBlock = cssBlock(".sidebar-runtime .icon-button");
  const smallBlock = lastCssBlock(".sidebar-runtime small");

  assert.ok(runtimeBlock, "expected sidebar runtime CSS block");
  assert.ok(textBlock, "expected sidebar runtime text CSS block");
  assert.ok(buttonBlock, "expected sidebar runtime button CSS block");
  assert.ok(smallBlock, "expected sidebar runtime small CSS block");
  assert.match(runtimeBlock, /align-items:\s*center/);
  assert.match(runtimeBlock, /row-gap:\s*2px/);
  assert.match(textBlock, /gap:\s*1px/);
  assert.match(buttonBlock, /grid-row:\s*1\s*\/\s*3/);
  assert.match(buttonBlock, /align-self:\s*center/);
  assert.match(smallBlock, /grid-column:\s*1/);
});

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

test("proxy table fits within the minimum window width", async () => {
  const [tauriConfig, css] = await Promise.all([
    readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
    readFile(new URL("../src/styles.css", import.meta.url), "utf8"),
  ]);

  const minWindowWidth = JSON.parse(tauriConfig).app.windows[0].minWidth;
  const sidebarMaxWidth = readLastPx(css, /\.app-shell[\s\S]*?grid-template-columns:\s*minmax\(\s*\d+px,\s*(\d+)px\)/);
  const workbenchPadding = readLastPx(css, /\.workbench[\s\S]*?padding:\s*(\d+)px/);
  const availableTableWidth = minWindowWidth - sidebarMaxWidth - workbenchPadding * 2;

  const tableMinWidth = readLastPx(css, /\.proxy-table-header,\s*\n\.proxy-table-row\s*\{[\s\S]*?min-width:\s*(\d+)px/);
  assert.ok(
    tableMinWidth <= availableTableWidth,
    `proxy table min-width ${tableMinWidth}px must fit available ${availableTableWidth}px`,
  );

  const tableGridBlock = readMatch(
    css,
    /\.panel-header,\s*\n\.proxy-table-header,\s*\n\.proxy-table-row,\s*\n\.proxy-table-empty\s*\{([\s\S]*?)\n\}/,
  );
  const template = readMatch(tableGridBlock, /grid-template-columns:\s*([^;]+);/);
  const gap = readLastPx(tableGridBlock, /gap:\s*(\d+)px/);
  const horizontalPadding = readLastPx(tableGridBlock, /padding:\s*\d+px\s+(\d+)px/);
  const minimumGridWidth =
    sumGridMinimums(template) + 5 * gap + 2 * horizontalPadding;
  assert.ok(
    minimumGridWidth <= availableTableWidth,
    `proxy table grid minimum ${minimumGridWidth}px must fit available ${availableTableWidth}px`,
  );
});

function readLastPx(source, pattern) {
  return Number(readMatch(source, pattern));
}

function readMatch(source, pattern) {
  const match = source.match(pattern);
  assert.ok(match, `pattern not found: ${pattern}`);
  return match[1];
}

function sumGridMinimums(template) {
  const fixedPixels = [...template.matchAll(/(?<!minmax\()\b(\d+)px\b/g)].map(
    ([, value]) => Number(value),
  );
  const minmaxPixels = [...template.matchAll(/minmax\(\s*(\d+)px/g)].map(
    ([, value]) => Number(value),
  );
  return [...fixedPixels, ...minmaxPixels].reduce((sum, value) => sum + value, 0);
}

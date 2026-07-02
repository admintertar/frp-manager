import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import ts from "typescript";

const source = await readFile(
  new URL("../src/lib/portInput.ts", import.meta.url),
  "utf8",
);
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ES2020,
    target: ts.ScriptTarget.ES2020,
  },
});
const moduleUrl = `data:text/javascript;base64,${Buffer.from(compiled.outputText).toString("base64")}`;
const { sanitizePortInput } = await import(moduleUrl);

test("port input only keeps digits", () => {
  assert.equal(sanitizePortInput("70a0-1"), "7001");
});

test("port input limits entry length to five digits", () => {
  assert.equal(sanitizePortInput("123456789"), "12345");
});

test("profile and proxy editors sanitize all port fields while typing", async () => {
  const profile = await readFile(
    new URL("../src/components/ProfileEditor.tsx", import.meta.url),
    "utf8",
  );
  const proxy = await readFile(
    new URL("../src/components/ProxyEditor.tsx", import.meta.url),
    "utf8",
  );

  assert.match(profile, /sanitizePortInput\(event\.target\.value\)/);
  assert.match(proxy, /setLocalPort\(sanitizePortInput\(event\.target\.value\)\)/);
  assert.match(proxy, /setRemotePort\(sanitizePortInput\(event\.target\.value\)\)/);
});

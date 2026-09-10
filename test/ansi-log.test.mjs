import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import ts from "typescript";

const source = await readFile(
  new URL("../src/lib/ansiLog.ts", import.meta.url),
  "utf8",
);
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ES2020,
    target: ts.ScriptTarget.ES2020,
  },
});
const moduleUrl = `data:text/javascript;base64,${Buffer.from(compiled.outputText).toString("base64")}`;
const { parseAnsiLogLine } = await import(moduleUrl);

test("ansi log parser keeps blue bold frpc info output", () => {
  assert.deepEqual(
    parseAnsiLogLine("\u001b[1;34m2026-07-01 [I] connected\u001b[0m plain"),
    [
      {
        text: "2026-07-01 [I] connected",
        color: "blue",
        bold: true,
      },
      {
        text: " plain",
      },
    ],
  );
});

test("ansi log parser maps warning and error colors", () => {
  assert.deepEqual(
    parseAnsiLogLine("\u001b[33mwarn\u001b[0m \u001b[31merror\u001b[0m"),
    [
      { text: "warn", color: "yellow" },
      { text: " " },
      { text: "error", color: "red" },
    ],
  );
});

test("log rendering preserves ansi input for the frontend parser", async () => {
  const workbench = await readFile(
    new URL("../src/components/ProfileWorkbench.tsx", import.meta.url),
    "utf8",
  );
  const commands = await readFile(
    new URL("../src-tauri/src/commands.rs", import.meta.url),
    "utf8",
  );

  assert.match(workbench, /parseAnsiLogLine/);
  assert.match(workbench, /log-token/);
  assert.doesNotMatch(workbench, /stripAnsiCodes/);

  // The log read path is bounded by size, but the bytes handed to the frontend
  // must stay raw so the ANSI parser can still colour them.
  assert.match(commands, /Ok\(log_store::read_tail\(&log_path/);
  assert.doesNotMatch(commands, /clean_log_output\([^)]*read_tail/);
  assert.doesNotMatch(commands, /clean_log_output\(&fs::read_to_string\(log_path\)\?\)/);
});

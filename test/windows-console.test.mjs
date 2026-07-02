import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const mainSource = readFileSync("src-tauri/src/main.rs", "utf8");
const processManagerSource = readFileSync("src-tauri/src/process_manager.rs", "utf8");

test("Windows builds use the GUI subsystem instead of opening an app console", () => {
  assert.match(mainSource, /windows_subsystem\s*=\s*"windows"/);
});

test("Windows frpc child processes are launched without their own console window", () => {
  assert.match(processManagerSource, /WINDOWS_CREATE_NO_WINDOW\s*:\s*u32\s*=\s*0x08000000/);
  assert.match(processManagerSource, /\.creation_flags\(\s*WINDOWS_CREATE_NO_WINDOW\s*\)/);
});

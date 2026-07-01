import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AddProxyInput,
  CreateProfileInput,
  Profile,
  ProfileSummary,
  RuntimeUpdateCheck,
  UpdateProxyInput,
  UpdateProfileInput,
} from "../types";

export const PROFILE_STATE_CHANGED_EVENT = "profile-state-changed";

export function listenProfileStateChanged(
  handler: () => void,
): Promise<(() => void) | undefined> {
  if (!hasTauriRuntime()) return Promise.resolve(undefined);
  return listen(PROFILE_STATE_CHANGED_EVENT, () => handler());
}

export function listProfiles(): Promise<ProfileSummary[]> {
  return invokeOrFallback("list_profiles", undefined, []);
}

export function getProfile(profileId: string): Promise<Profile> {
  return invokeOrFallback("get_profile", { profileId });
}

export function importProfileFromText(
  name: string,
  toml: string,
): Promise<ProfileSummary> {
  return invokeOrFallback("import_profile_from_text", { name, toml });
}

export function createProfile(
  input: CreateProfileInput,
): Promise<ProfileSummary> {
  return invokeOrFallback("create_profile", { input });
}

export function updateProfile(
  profileId: string,
  input: UpdateProfileInput,
): Promise<void> {
  return invokeOrFallback("update_profile", { profileId, input });
}

export function deleteProfile(profileId: string): Promise<void> {
  return invokeOrFallback("delete_profile", { profileId });
}

export function saveProfile(profileId: string, rawToml: string): Promise<void> {
  return invokeOrFallback("save_profile", { profileId, rawToml });
}

export function startProfile(profileId: string): Promise<void> {
  return invokeOrFallback("start_profile", { profileId });
}

export function stopProfile(profileId: string): Promise<void> {
  return invokeOrFallback("stop_profile", { profileId });
}

export function toggleProxy(
  profileId: string,
  proxyName: string,
  enabled: boolean,
): Promise<void> {
  return invokeOrFallback("toggle_proxy", { profileId, proxyName, enabled });
}

export function addProxy(
  profileId: string,
  input: AddProxyInput,
): Promise<void> {
  return invokeOrFallback("add_proxy", { profileId, input });
}

export function updateProxy(
  profileId: string,
  proxyName: string,
  input: UpdateProxyInput,
): Promise<void> {
  return invokeOrFallback("update_proxy", { profileId, proxyName, input });
}

export function deleteProxy(
  profileId: string,
  proxyName: string,
): Promise<void> {
  return invokeOrFallback("delete_proxy", { profileId, proxyName });
}

export function readProfileLogs(profileId: string): Promise<string> {
  return invokeOrFallback("read_profile_logs", { profileId }, "");
}

export function getRuntimeInfo(): Promise<string> {
  return invokeOrFallback("get_runtime_info", undefined, "0.69.1");
}

export function checkRuntimeUpdate(): Promise<RuntimeUpdateCheck> {
  return invokeOrFallback("check_runtime_update", undefined, {
    currentVersion: "0.69.1",
    latestVersion: "0.69.1",
    updateAvailable: false,
    assetName: "frp_0.69.1_darwin_arm64.tar.gz",
  });
}

function invokeOrFallback<T>(
  command: string,
  args?: Record<string, unknown>,
  browserFallback?: T,
): Promise<T> {
  if (!hasTauriRuntime()) {
    if (browserFallback !== undefined) return Promise.resolve(browserFallback);
    return Promise.reject(new Error("Tauri runtime is not available"));
  }
  return invoke<T>(command, args);
}

function hasTauriRuntime() {
  return (
    typeof window !== "undefined" &&
    "__TAURI_INTERNALS__" in window &&
    window.__TAURI_INTERNALS__ !== undefined
  );
}

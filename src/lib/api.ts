import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AddProxyInput,
  AppUpdateCheck,
  AppUpdateInstall,
  CreateProfileInput,
  Profile,
  ProfileSummary,
  RuntimeStatus,
  RuntimeUpdateCheck,
  UpdateProxyInput,
  UpdateProfileInput,
} from "../types";

export const PROFILE_STATE_CHANGED_EVENT = "profile-state-changed";
export const APP_UPDATE_CHECK_REQUESTED_EVENT = "app-update-check-requested";

export function listenProfileStateChanged(
  handler: () => void,
): Promise<(() => void) | undefined> {
  if (!hasTauriRuntime()) return Promise.resolve(undefined);
  return listen(PROFILE_STATE_CHANGED_EVENT, () => handler());
}

export function listenAppUpdateCheckRequested(
  handler: () => void,
): Promise<(() => void) | undefined> {
  if (!hasTauriRuntime()) return Promise.resolve(undefined);
  return listen(APP_UPDATE_CHECK_REQUESTED_EVENT, () => handler());
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

export function setProfileAutoStart(
  profileId: string,
  autoStart: boolean,
): Promise<void> {
  return invokeOrFallback("set_profile_auto_start", { profileId, autoStart });
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

export function readAppLog(): Promise<string> {
  return invokeOrFallback("read_app_log", undefined, "");
}

export function getRuntimeInfo(): Promise<string> {
  return invokeOrFallback("get_runtime_info", undefined, "not installed");
}

export function getRuntimeStatus(): Promise<RuntimeStatus> {
  return invokeOrFallback("get_runtime_status", undefined, {
    installed: false,
    currentVersion: null,
    runtimePath: null,
    platform: {
      os: "darwin",
      arch: "arm64",
      executableName: "frpc",
    },
  });
}

export function checkRuntimeUpdate(): Promise<RuntimeUpdateCheck> {
  return invokeOrFallback("check_runtime_update", undefined, {
    currentVersion: null,
    latestVersion: "0.69.1",
    updateAvailable: true,
    assetName: "frp_0.69.1_darwin_arm64.tar.gz",
    installed: false,
    runtimePath: null,
    platform: {
      os: "darwin",
      arch: "arm64",
      executableName: "frpc",
    },
  });
}

export function installRuntime(): Promise<RuntimeStatus> {
  return invokeOrFallback("install_runtime");
}

export function checkAppUpdate(): Promise<AppUpdateCheck> {
  return invokeOrFallback("check_app_update", undefined, {
    currentVersion: "0.0.3",
    latestVersion: "0.0.3",
    updateAvailable: false,
    releaseUrl: "https://github.com/admintertar/frp-manager/releases",
    assetName: "FRP-Manager_0.0.3_darwin_aarch64.dmg",
    downloadUrl:
      "https://github.com/admintertar/frp-manager/releases/download/v0.0.3/FRP-Manager_0.0.3_darwin_aarch64.dmg",
    installerPath: "",
    downloaded: false,
  });
}

export function downloadAppUpdate(): Promise<AppUpdateInstall> {
  return invokeOrFallback("download_app_update", undefined, {
    assetName: "FRP-Manager_0.0.3_darwin_aarch64.dmg",
    installerPath: "",
  });
}

export function openAppUpdateInstaller(): Promise<AppUpdateInstall> {
  return invokeOrFallback("open_app_update_installer", undefined, {
    assetName: "FRP-Manager_0.0.3_darwin_aarch64.dmg",
    installerPath: "",
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

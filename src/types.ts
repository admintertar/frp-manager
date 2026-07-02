export type RuntimeState =
  | "stopped"
  | "starting"
  | "running"
  | "reloading"
  | "degraded"
  | "failed";

export type ProxyType = "http" | "https" | "tcp" | "udp";

export type ProfileAuthMethod = "token" | "oidc";

export interface CreateProfileInput {
  profileName: string;
  serverAddr: string;
  serverPort: number;
  authMethod: ProfileAuthMethod;
  authToken: string | null;
  oidcClientId: string | null;
  oidcClientSecret: string | null;
  oidcAudience: string | null;
  oidcTokenEndpointUrl: string | null;
}

export type UpdateProfileInput = CreateProfileInput;

export interface AddProxyInput {
  name: string;
  proxyType: ProxyType;
  localIp: string | null;
  localPort: number | null;
  remotePort: number | null;
  subdomain: string | null;
  customDomains: string[];
}

export type UpdateProxyInput = AddProxyInput;

export interface ProfileSummary {
  id: string;
  displayName: string;
  serverAddr: string;
  serverPort: number;
  proxyCount: number;
  runtimeState: RuntimeState;
  runtimePid: number | null;
  runtimeStartedAt: string | null;
}

export interface Profile {
  id: string;
  displayName: string;
  serverAddr: string;
  serverPort: number;
  authMethod: string | null;
  authToken: string | null;
  adminPort: number | null;
  proxies: ProxyConfig[];
  rawToml: string;
  meta: ProfileMeta;
}

export interface ProfileMeta {
  id: string;
  displayName: string;
  createdAt: string;
  updatedAt: string;
  autoStart: boolean;
  lastRuntimeVersion: string | null;
}

export interface ProxyConfig {
  name: string;
  proxyType: ProxyType;
  enabled: boolean;
  localIp: string | null;
  localPort: number | null;
  remotePort: number | null;
  subdomain: string | null;
  customDomains: string[];
}

export interface RuntimeUpdateCheck {
  currentVersion: string | null;
  latestVersion: string;
  updateAvailable: boolean;
  assetName: string | null;
  installed: boolean;
  runtimePath: string | null;
  platform: RuntimePlatform;
}

export interface RuntimePlatform {
  os: string;
  arch: string;
  executableName: string;
}

export interface RuntimeStatus {
  installed: boolean;
  currentVersion: string | null;
  runtimePath: string | null;
  platform: RuntimePlatform;
}

export interface AppUpdateCheck {
  currentVersion: string;
  latestVersion: string;
  updateAvailable: boolean;
  releaseUrl: string;
}

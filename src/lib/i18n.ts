import { useMemo, useSyncExternalStore } from "react";

export type Locale = "en" | "zh-CN";

/** Languages offered in the UI, in display order. */
export const LOCALES: Locale[] = ["zh-CN", "en"];

export const DEFAULT_LOCALE: Locale = "en";

/**
 * English is the reference table: every other locale must define the same keys,
 * and `t()` is typed against it so a typo is a compile error.
 */
const en = {
  "common.cancel": "Cancel",
  "common.close": "Close",
  "common.save": "Save",
  "common.saving": "Saving",
  "common.create": "Create",
  "common.creating": "Creating",
  "common.edit": "Edit",
  "common.delete": "Delete",
  "common.open": "Open",
  "common.download": "Download",
  "common.downloading": "Downloading",
  "common.downloadingEllipsis": "Downloading...",
  "common.update": "Update",
  "common.later": "Later",
  "common.checking": "Checking",
  "common.unknown": "unknown",

  "sidebar.title": "Profiles",
  "sidebar.configured": "{count} configured",
  "sidebar.addProfile": "Add profile",
  "sidebar.startOnLaunch": "Start on launch",
  "sidebar.autoStartOn": "On",
  "sidebar.autoStartOff": "Off",
  "sidebar.runtime": "frpc runtime",
  "sidebar.runtimeSettings": "Runtime settings",
  "sidebar.autoBadge": "auto",
  "sidebar.autoBadgeTitle": "Starts when FRP Manager opens",
  "sidebar.proxyCountOne": "{count} proxy",
  "sidebar.proxyCountOther": "{count} proxies",
  "sidebar.menuLabel": "{name} actions",
  "sidebar.language": "Language",
  "sidebar.switchLanguage": "Switch to {language}",

  "workbench.stop": "Stop",
  "workbench.start": "Start",
  "workbench.reload": "Reload",
  "workbench.refresh": "Refresh profiles",
  "workbench.installRuntimeFirst": "Install frpc runtime before starting profiles",
  "workbench.authMethod": "{method} auth",
  "workbench.authNotSet": "auth not set",
  "workbench.loadingProfile": "loading profile",
  "workbench.importHint": "Import a frpc profile to begin.",
  "workbench.detail": "serverAddr: {addr} · serverPort: {port} · {auth}",

  "metrics.status": "Status",
  "metrics.proxies": "Proxies",
  "metrics.pid": "PID",
  "metrics.uptime": "Uptime",
  "metrics.active": "{count} active",
  "metrics.configured": "{count} configured",

  "empty.noProfileTitle": "No Profile selected",
  "empty.noProfileBody": "Create a profile to start managing frpc.",
  "empty.loadingTitle": "Loading profile",
  "empty.loadingBody": "Preparing {name}.",

  "table.title": "Proxies",
  "table.addProxy": "Add Proxy",
  "table.columnName": "Name",
  "table.columnType": "Type",
  "table.columnLocal": "Local",
  "table.columnRemote": "Remote",
  "table.columnStatus": "Status",
  "table.columnActions": "Actions",
  "table.online": "online",
  "table.off": "off",
  "table.enable": "Enable {name}",
  "table.editProxy": "Edit {name}",
  "table.deleteProxy": "Delete {name}",
  "table.empty": "This Profile has no proxies.",

  "log.waiting": "[I] waiting for frpc log output",

  "profileEditor.editTitle": "Edit frpc Profile",
  "profileEditor.createTitle": "New frpc Profile",
  "profileEditor.profileName": "Profile name",
  "profileEditor.serverAddr": "Server address",
  "profileEditor.serverPort": "Server port",
  "profileEditor.authMethod": "Auth method",
  "profileEditor.authToken": "Auth token",

  "proxyEditor.editTitle": "Edit Proxy",
  "proxyEditor.addTitle": "Add Proxy",
  "proxyEditor.proxyName": "Proxy name",
  "proxyEditor.mappingAddress": "Mapping address",
  "proxyEditor.type": "Type",
  "proxyEditor.proxyTypeLabel": "Proxy type",
  "proxyEditor.localIp": "Local IP",
  "proxyEditor.localPort": "Local port",
  "proxyEditor.remotePort": "Remote port",
  "proxyEditor.subdomain": "Subdomain",
  "proxyEditor.customDomains": "Custom domains",
  "proxyEditor.addAction": "Add Proxy",
  "proxyEditor.adding": "Adding",

  "runtimeSettings.title": "frpc Runtime",
  "runtimeSettings.close": "Close runtime settings",
  "runtimeSettings.updateAvailable": "Update available",
  "runtimeSettings.upToDate": "Runtime is current",
  "runtimeSettings.installed": "Runtime installed",
  "runtimeSettings.notInstalled": "frpc runtime not installed",
  "runtimeSettings.notInstalledShort": "not installed",
  "runtimeSettings.fieldCurrent": "Current",
  "runtimeSettings.fieldLatest": "Latest",
  "runtimeSettings.fieldPlatform": "Platform",
  "runtimeSettings.fieldAsset": "Asset",
  "runtimeSettings.fieldPath": "Path",
  "runtimeSettings.openFolder": "Open Folder",
  "runtimeSettings.checkUpdates": "Check Updates",

  "appUpdate.available": "New version available",
  "appUpdate.upToDate": "FRP Manager is up to date",
  "appUpdate.noNewer": "No newer release was found",
  "appUpdate.close": "Close update prompt",
  "appUpdate.current": "Current {version}",
  "appUpdate.latest": "Latest {version}",
  "appUpdate.ignoreVersion": "Ignore this version",

  "confirm.deleteProxy": "Delete proxy \"{name}\"?",
  "confirm.deleteProfile": "Delete profile \"{name}\"?",

  "state.stopped": "stopped",
  "state.starting": "starting",
  "state.running": "running",
  "state.reloading": "reloading",
  "state.degraded": "degraded",
  "state.failed": "failed",
} as const;

export type MessageKey = keyof typeof en;

const zhCN: Record<MessageKey, string> = {
  "common.cancel": "取消",
  "common.close": "关闭",
  "common.save": "保存",
  "common.saving": "保存中",
  "common.create": "创建",
  "common.creating": "创建中",
  "common.edit": "编辑",
  "common.delete": "删除",
  "common.open": "打开",
  "common.download": "下载",
  "common.downloading": "下载中",
  "common.downloadingEllipsis": "下载中…",
  "common.update": "更新",
  "common.later": "稍后",
  "common.checking": "检查中",
  "common.unknown": "未知",

  "sidebar.title": "配置",
  "sidebar.configured": "共 {count} 个",
  "sidebar.addProfile": "新增配置",
  "sidebar.startOnLaunch": "启动时自动运行",
  "sidebar.autoStartOn": "已开启",
  "sidebar.autoStartOff": "已关闭",
  "sidebar.runtime": "frpc 运行时",
  "sidebar.runtimeSettings": "运行时设置",
  "sidebar.autoBadge": "自启",
  "sidebar.autoBadgeTitle": "FRP Manager 打开时自动启动",
  "sidebar.proxyCountOne": "{count} 个映射",
  "sidebar.proxyCountOther": "{count} 个映射",
  "sidebar.menuLabel": "{name} 的操作",
  "sidebar.language": "语言",
  "sidebar.switchLanguage": "切换到{language}",

  "workbench.stop": "停止",
  "workbench.start": "启动",
  "workbench.reload": "重载",
  "workbench.refresh": "刷新配置",
  "workbench.installRuntimeFirst": "请先安装 frpc 运行时再启动配置",
  "workbench.authMethod": "{method} 认证",
  "workbench.authNotSet": "未设置认证",
  "workbench.loadingProfile": "正在加载配置",
  "workbench.importHint": "导入一个 frpc 配置开始使用。",
  "workbench.detail": "serverAddr: {addr} · serverPort: {port} · {auth}",

  "metrics.status": "状态",
  "metrics.proxies": "映射",
  "metrics.pid": "进程号",
  "metrics.uptime": "运行时长",
  "metrics.active": "{count} 个已启用",
  "metrics.configured": "{count} 个已配置",

  "empty.noProfileTitle": "未选择配置",
  "empty.noProfileBody": "创建一个配置即可开始管理 frpc。",
  "empty.loadingTitle": "正在加载配置",
  "empty.loadingBody": "正在准备 {name}。",

  "table.title": "映射",
  "table.addProxy": "添加映射",
  "table.columnName": "名称",
  "table.columnType": "类型",
  "table.columnLocal": "本地",
  "table.columnRemote": "远端",
  "table.columnStatus": "状态",
  "table.columnActions": "操作",
  "table.online": "在线",
  "table.off": "关闭",
  "table.enable": "启用 {name}",
  "table.editProxy": "编辑 {name}",
  "table.deleteProxy": "删除 {name}",
  "table.empty": "该配置还没有映射。",

  "log.waiting": "[I] 等待 frpc 日志输出",

  "profileEditor.editTitle": "编辑 frpc 配置",
  "profileEditor.createTitle": "新建 frpc 配置",
  "profileEditor.profileName": "配置名称",
  "profileEditor.serverAddr": "服务端地址",
  "profileEditor.serverPort": "服务端端口",
  "profileEditor.authMethod": "认证方式",
  "profileEditor.authToken": "认证 token",

  "proxyEditor.editTitle": "编辑映射",
  "proxyEditor.addTitle": "添加映射",
  "proxyEditor.proxyName": "映射名称",
  "proxyEditor.mappingAddress": "映射地址",
  "proxyEditor.type": "类型",
  "proxyEditor.proxyTypeLabel": "映射类型",
  "proxyEditor.localIp": "本地 IP",
  "proxyEditor.localPort": "本地端口",
  "proxyEditor.remotePort": "远端端口",
  "proxyEditor.subdomain": "子域名",
  "proxyEditor.customDomains": "自定义域名",
  "proxyEditor.addAction": "添加映射",
  "proxyEditor.adding": "添加中",

  "runtimeSettings.title": "frpc 运行时",
  "runtimeSettings.close": "关闭运行时设置",
  "runtimeSettings.updateAvailable": "有新版本可用",
  "runtimeSettings.upToDate": "运行时已是最新",
  "runtimeSettings.installed": "运行时已安装",
  "runtimeSettings.notInstalled": "尚未安装 frpc 运行时",
  "runtimeSettings.notInstalledShort": "未安装",
  "runtimeSettings.fieldCurrent": "当前版本",
  "runtimeSettings.fieldLatest": "最新版本",
  "runtimeSettings.fieldPlatform": "平台",
  "runtimeSettings.fieldAsset": "安装包",
  "runtimeSettings.fieldPath": "路径",
  "runtimeSettings.openFolder": "打开目录",
  "runtimeSettings.checkUpdates": "检查更新",

  "appUpdate.available": "有新版本可用",
  "appUpdate.upToDate": "FRP Manager 已是最新版本",
  "appUpdate.noNewer": "没有找到更新的版本",
  "appUpdate.close": "关闭更新提示",
  "appUpdate.current": "当前 {version}",
  "appUpdate.latest": "最新 {version}",
  "appUpdate.ignoreVersion": "忽略此版本",

  "confirm.deleteProxy": "确定删除映射「{name}」？",
  "confirm.deleteProfile": "确定删除配置「{name}」？",

  "state.stopped": "已停止",
  "state.starting": "启动中",
  "state.running": "运行中",
  "state.reloading": "重载中",
  "state.degraded": "降级",
  "state.failed": "失败",
};

const messages: Record<Locale, Record<MessageKey, string>> = {
  en,
  "zh-CN": zhCN,
};

export type MessageParams = Record<string, string | number>;

function interpolate(template: string, params?: MessageParams): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (match, key: string) => {
    const value = params[key];
    return value === undefined ? match : String(value);
  });
}

/** Translate `key` for `locale`, substituting `{name}` placeholders. */
export function translate(
  locale: Locale,
  key: MessageKey,
  params?: MessageParams,
): string {
  const table = messages[locale] ?? messages[DEFAULT_LOCALE];
  return interpolate(table[key] ?? messages[DEFAULT_LOCALE][key], params);
}

export type Translator = (key: MessageKey, params?: MessageParams) => string;

export function createTranslator(locale: Locale): Translator {
  return (key, params) => translate(locale, key, params);
}

/** Resolve a stored or browser-reported tag to one of the supported locales. */
export function resolveLocale(candidate?: string | null): Locale {
  if (!candidate) return DEFAULT_LOCALE;
  const normalized = candidate.trim().toLowerCase();
  if (normalized.startsWith("zh")) return "zh-CN";
  if (normalized.startsWith("en")) return "en";
  return DEFAULT_LOCALE;
}

/** Human-readable name of a locale, always written in that language. */
export function localeLabel(locale: Locale): string {
  return locale === "zh-CN" ? "中文" : "English";
}

export function otherLocale(locale: Locale): Locale {
  return locale === "zh-CN" ? "en" : "zh-CN";
}

/* ------------------------------------------------------------------ */
/* Tiny observable store so components re-render when the locale flips. */
/* ------------------------------------------------------------------ */

let currentLocale: Locale = DEFAULT_LOCALE;
const listeners = new Set<() => void>();

export function getLocale(): Locale {
  return currentLocale;
}

export function setLocale(next: Locale): void {
  if (next === currentLocale) return;
  currentLocale = next;
  listeners.forEach((listener) => listener());
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function useLocale(): Locale {
  return useSyncExternalStore(subscribe, getLocale, getLocale);
}

/** Translator bound to the active locale; re-renders on change. */
export function useTranslation(): Translator {
  const locale = useLocale();
  return useMemo(() => createTranslator(locale), [locale]);
}

/** Message keys of a locale, for tests that compare table completeness. */
export function messageKeys(locale: Locale): MessageKey[] {
  return Object.keys(messages[locale]).sort() as MessageKey[];
}

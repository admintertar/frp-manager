import { useEffect, useMemo, useRef, useState } from "react";
import { ProfileEditor } from "./components/ProfileEditor";
import { ProfileSidebar } from "./components/ProfileSidebar";
import { ProfileWorkbench } from "./components/ProfileWorkbench";
import { ProxyEditor } from "./components/ProxyEditor";
import { RuntimeSettings } from "./components/RuntimeSettings";
import {
  addProxy,
  createProfile,
  deleteProfile,
  deleteProxy,
  getProfile,
  getRuntimeStatus,
  listenProfileStateChanged,
  listProfiles,
  readProfileLogs,
  startProfile,
  stopProfile,
  toggleProxy,
  updateProfile,
  updateProxy,
} from "./lib/api";
import type {
  AddProxyInput,
  CreateProfileInput,
  Profile,
  ProfileSummary,
  ProxyConfig,
  RuntimeStatus,
  RuntimeState,
  UpdateProxyInput,
  UpdateProfileInput,
} from "./types";
import "./styles.css";

export default function App() {
  const [profiles, setProfiles] = useState<ProfileSummary[]>([]);
  const [profileDetail, setProfileDetail] = useState<Profile | undefined>();
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const [runtimeStatus, setRuntimeStatus] = useState<RuntimeStatus>(
    fallbackRuntimeStatus(),
  );
  const [importOpen, setImportOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [proxyOpen, setProxyOpen] = useState(false);
  const [editingProxy, setEditingProxy] = useState<ProxyConfig | null>(null);
  const [runtimeOpen, setRuntimeOpen] = useState(false);
  const [profileLogs, setProfileLogs] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [importError, setImportError] = useState<string | null>(null);
  const [editError, setEditError] = useState<string | null>(null);
  const [proxyError, setProxyError] = useState<string | null>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [busyProxyName, setBusyProxyName] = useState<string | null>(null);
  const selectedIdRef = useRef<string | undefined>(undefined);
  const pendingSelectionRef = useRef<string | undefined>(undefined);

  const selected = useMemo(
    () => profiles.find((profile) => profile.id === selectedId) ?? profiles[0],
    [profiles, selectedId],
  );
  const currentProfileDetail =
    profileDetail?.id === selected?.id ? profileDetail : undefined;
  const runtimeState: RuntimeState = selected?.runtimeState ?? "stopped";
  const runtimeLabel = runtimeStatus.installed
    ? runtimeStatus.currentVersion ?? "installed"
    : "not installed";

  async function fetchProfileDetail(profileId: string) {
    const [profile, logs] = await Promise.all([
      getProfile(profileId),
      readProfileLogs(profileId),
    ]);
    return { profile, logs };
  }

  async function loadProfileDetail(profileId: string | undefined) {
    if (!profileId) {
      setProfileDetail(undefined);
      setProfileLogs("");
      return;
    }
    try {
      const { profile, logs } = await fetchProfileDetail(profileId);
      if (profile.id === profileId && selectedIdRef.current === profileId) {
        setProfileDetail(profile);
        setProfileLogs(logs);
        setError(null);
      }
    } catch (err) {
      if (selectedIdRef.current === profileId) {
        setError(formatInvokeError(err));
      }
    }
  }

  async function refreshProfiles(preferredSelectedId?: string, silent = false) {
    if (!silent) setBusyAction("refresh");
    try {
      const next = await listProfiles();
      setProfiles(next);
      const nextSelectedId = reconcileSelectedId(
        preferredSelectedId ?? selectedIdRef.current,
        next,
      );
      selectedIdRef.current = nextSelectedId;
      setSelectedId(nextSelectedId);
      await loadProfileDetail(nextSelectedId);
      setError(null);
    } catch (err) {
      setError(formatInvokeError(err));
    } finally {
      if (!silent) {
        setBusyAction((current) => (current === "refresh" ? null : current));
      }
    }
  }

  async function refreshRuntimeInfo() {
    try {
      setRuntimeStatus(await getRuntimeStatus());
    } catch (err) {
      setError(formatInvokeError(err));
    }
  }

  useEffect(() => {
    void refreshProfiles();
    void refreshRuntimeInfo();
  }, []);

  useEffect(() => {
    const timer = window.setInterval(() => {
      void refreshProfiles(undefined, true);
    }, 2500);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void listenProfileStateChanged(() => {
      void refreshProfiles(selectedIdRef.current, true);
    })
      .then((dispose) => {
        if (disposed) {
          dispose?.();
          return;
        }
        unlisten = dispose;
      })
      .catch((err) => {
        if (!disposed) setError(formatInvokeError(err));
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    const requestedId = selected?.id;
    selectedIdRef.current = requestedId;
    if (!requestedId) {
      setProfileDetail(undefined);
      setProfileLogs("");
    }
  }, [selected?.id]);

  useEffect(() => {
    const id = selected?.id;
    if (!id) return;

    setBusyAction((current) => {
      if (current === `start:${id}` && runtimeState === "running") return null;
      if (current === `stop:${id}` && runtimeState !== "running") return null;
      return current;
    });
  }, [selected?.id, runtimeState]);

  async function handleCreateProfile(input: CreateProfileInput) {
    try {
      setImportError(null);
      const imported = await createProfile(input);
      selectedIdRef.current = imported.id;
      setSelectedId(imported.id);
      await refreshProfiles(imported.id);
    } catch (err) {
      setImportError(formatInvokeError(err));
      throw err;
    }
  }

  async function handleUpdateProfile(input: UpdateProfileInput) {
    if (!selected?.id) return;
    try {
      setEditError(null);
      await updateProfile(selected.id, input);
      await refreshProfiles(selected.id);
      await loadProfileDetail(selected.id);
    } catch (err) {
      setEditError(formatInvokeError(err));
      throw err;
    }
  }

  async function openProfileEditor(profileId: string | undefined) {
    if (!profileId) return;
    pendingSelectionRef.current = profileId;
    setEditError(null);
    try {
      const detail = await fetchProfileDetail(profileId);
      if (pendingSelectionRef.current !== profileId) return;
      pendingSelectionRef.current = undefined;
      selectedIdRef.current = profileId;
      setSelectedId(profileId);
      setProfileDetail(detail.profile);
      setProfileLogs(detail.logs);
      setError(null);
    } catch (err) {
      if (pendingSelectionRef.current === profileId) {
        pendingSelectionRef.current = undefined;
        setError(formatInvokeError(err));
      }
      return;
    }
    setEditOpen(true);
  }

  async function handleAddProxy(input: AddProxyInput) {
    if (!selected?.id) return;
    try {
      setProxyError(null);
      await addProxy(selected.id, input);
      await refreshProfiles(selected.id);
      await loadProfileDetail(selected.id);
    } catch (err) {
      setProxyError(formatInvokeError(err));
      throw err;
    }
  }

  async function handleSaveProxy(input: AddProxyInput | UpdateProxyInput) {
    if (!selected?.id) return;
    if (editingProxy) {
      await handleUpdateProxy(editingProxy.name, input);
    } else {
      await handleAddProxy(input);
    }
  }

  async function handleUpdateProxy(proxyName: string, input: UpdateProxyInput) {
    if (!selected?.id) return;
    try {
      setProxyError(null);
      await updateProxy(selected.id, proxyName, input);
      await refreshProfiles(selected.id);
      await loadProfileDetail(selected.id);
    } catch (err) {
      setProxyError(formatInvokeError(err));
      throw err;
    }
  }

  async function handleSelectProfile(profileId: string) {
    if (profileId === selected?.id) {
      pendingSelectionRef.current = undefined;
      return;
    }
    pendingSelectionRef.current = profileId;
    try {
      const detail = await fetchProfileDetail(profileId);
      if (pendingSelectionRef.current !== profileId) return;
      pendingSelectionRef.current = undefined;
      selectedIdRef.current = profileId;
      setSelectedId(profileId);
      setProfileDetail(detail.profile);
      setProfileLogs(detail.logs);
      setError(null);
    } catch (err) {
      if (pendingSelectionRef.current === profileId) {
        pendingSelectionRef.current = undefined;
        setError(formatInvokeError(err));
      }
    }
  }

  async function handleStart(profileId: string) {
    setBusyAction(`start:${profileId}`);
    try {
      await startProfile(profileId);
      await refreshProfiles(profileId, true);
    } catch (err) {
      setError(formatInvokeError(err));
    } finally {
      setBusyAction((current) =>
        current === `start:${profileId}` ? null : current,
      );
    }
  }

  async function handleStop(profileId: string) {
    setBusyAction(`stop:${profileId}`);
    try {
      await stopProfile(profileId);
      await refreshProfiles(profileId, true);
    } catch (err) {
      setError(formatInvokeError(err));
    } finally {
      setBusyAction((current) =>
        current === `stop:${profileId}` ? null : current,
      );
    }
  }

  async function handleReload(profileId: string) {
    const profile = profiles.find((profile) => profile.id === profileId);
    if (profile?.runtimeState !== "running") return;
    setBusyAction(`reload:${profileId}`);
    try {
      await stopProfile(profileId);
      await startProfile(profileId);
      await refreshProfiles(profileId, true);
      setError(null);
    } catch (err) {
      setError(formatInvokeError(err));
    } finally {
      setBusyAction((current) =>
        current === `reload:${profileId}` ? null : current,
      );
    }
  }

  async function handleToggleProxy(
    profileId: string,
    proxyName: string,
    enabled: boolean,
  ) {
    setBusyProxyName(proxyName);
    try {
      await toggleProxy(profileId, proxyName, enabled);
      await refreshProfiles();
      setError(null);
    } catch (err) {
      setError(formatInvokeError(err));
    } finally {
      setBusyProxyName((current) => (current === proxyName ? null : current));
    }
  }

  async function handleDeleteProxy(profileId: string, proxyName: string) {
    const confirmed = window.confirm(`Delete proxy "${proxyName}"?`);
    if (!confirmed) return;
    setBusyProxyName(proxyName);
    try {
      await deleteProxy(profileId, proxyName);
      await refreshProfiles(profileId);
      await loadProfileDetail(profileId);
      setError(null);
    } catch (err) {
      setError(formatInvokeError(err));
    } finally {
      setBusyProxyName((current) => (current === proxyName ? null : current));
    }
  }

  async function handleDeleteProfile(profileId: string) {
    const profile = profiles.find((profile) => profile.id === profileId);
    const confirmed = window.confirm(
      `Delete profile "${profile?.displayName ?? profileId}"?`,
    );
    if (!confirmed) return;

    setBusyAction(`delete:${profileId}`);
    try {
      await deleteProfile(profileId);
      if (pendingSelectionRef.current === profileId) {
        pendingSelectionRef.current = undefined;
      }
      if (selectedIdRef.current === profileId) {
        selectedIdRef.current = undefined;
        setSelectedId(undefined);
        setProfileDetail(undefined);
        setProfileLogs("");
      }
      setEditOpen(false);
      await refreshProfiles();
      setError(null);
    } catch (err) {
      setError(formatInvokeError(err));
    } finally {
      setBusyAction((current) =>
        current === `delete:${profileId}` ? null : current,
      );
    }
  }

  return (
    <main className="app-shell">
      <ProfileSidebar
        profiles={profiles}
        selectedId={selected?.id}
        runtimeVersion={runtimeLabel}
        onSelect={handleSelectProfile}
        onEditProfile={(profileId) => void openProfileEditor(profileId)}
        onDeleteProfile={(profileId) => void handleDeleteProfile(profileId)}
        onImport={() => {
          setImportError(null);
          setImportOpen(true);
        }}
        onOpenRuntimeSettings={() => setRuntimeOpen(true)}
      />
      <ProfileWorkbench
        profile={currentProfileDetail}
        selectedProfile={selected}
        runtimeState={runtimeState}
        runtimePid={selected?.runtimePid}
        runtimeStartedAt={selected?.runtimeStartedAt}
        runtimeInstalled={runtimeStatus.installed}
        logs={profileLogs}
        error={error}
        busyAction={busyAction}
        busyProxyName={busyProxyName}
        onRefresh={refreshProfiles}
        onReload={handleReload}
        onEdit={() => void openProfileEditor(selected?.id)}
        onAddProxy={() => {
          setEditingProxy(null);
          setProxyError(null);
          setProxyOpen(true);
        }}
        onEditProxy={(proxy) => {
          setEditingProxy(proxy);
          setProxyError(null);
          setProxyOpen(true);
        }}
        onDeleteProxy={handleDeleteProxy}
        onStart={handleStart}
        onStop={handleStop}
        onToggleProxy={handleToggleProxy}
      />
      <ProfileEditor
        open={importOpen}
        mode="create"
        error={importError}
        onClose={() => setImportOpen(false)}
        onSubmit={handleCreateProfile}
      />
      <ProfileEditor
        open={editOpen}
        mode="edit"
        profile={currentProfileDetail}
        error={editError}
        onClose={() => setEditOpen(false)}
        onSubmit={handleUpdateProfile}
      />
      <ProxyEditor
        open={proxyOpen}
        mode={editingProxy ? "edit" : "add"}
        serverAddr={currentProfileDetail?.serverAddr}
        proxy={editingProxy}
        error={proxyError}
        onClose={() => {
          setProxyOpen(false);
          setEditingProxy(null);
        }}
        onSubmit={handleSaveProxy}
      />
      <RuntimeSettings
        open={runtimeOpen}
        status={runtimeStatus}
        onClose={() => setRuntimeOpen(false)}
        onStatusChange={setRuntimeStatus}
      />
    </main>
  );
}

function fallbackRuntimeStatus(): RuntimeStatus {
  return {
    installed: false,
    currentVersion: null,
    runtimePath: null,
    platform: {
      os: "darwin",
      arch: "arm64",
      executableName: "frpc",
    },
  };
}

function reconcileSelectedId(
  current: string | undefined,
  profiles: ProfileSummary[],
): string | undefined {
  if (current && profiles.some((profile) => profile.id === current)) {
    return current;
  }
  return profiles[0]?.id;
}

function formatInvokeError(err: unknown): string {
  if (err instanceof Error) {
    return err.message;
  }
  if (typeof err === "object" && err !== null && "message" in err) {
    const message = (err as { message?: unknown }).message;
    if (typeof message === "string") return message;
  }
  return String(err);
}

import { useEffect, useMemo, useRef, useState } from "react";
import type { CreateProfileInput, Profile, ProfileAuthMethod } from "../types";

interface Props {
  open: boolean;
  mode: "create" | "edit";
  profile?: Profile;
  error: string | null;
  onClose: () => void;
  onSubmit: (input: CreateProfileInput) => Promise<void>;
}

export function ProfileEditor({
  open,
  mode,
  profile,
  error,
  onClose,
  onSubmit,
}: Props) {
  const [profileName, setProfileName] = useState("");
  const [serverAddr, setServerAddr] = useState("");
  const [serverPort, setServerPort] = useState("7000");
  const [authMethod, setAuthMethod] = useState<ProfileAuthMethod>("token");
  const [authToken, setAuthToken] = useState("");
  const [oidcClientId, setOidcClientId] = useState("");
  const [oidcClientSecret, setOidcClientSecret] = useState("");
  const [oidcAudience, setOidcAudience] = useState("");
  const [oidcTokenEndpointUrl, setOidcTokenEndpointUrl] = useState("");
  const [busy, setBusy] = useState(false);
  const nameInputRef = useRef<HTMLInputElement>(null);
  const busyRef = useRef(false);
  const onCloseRef = useRef(onClose);
  const isEditing = mode === "edit";

  const portNumber = Number(serverPort);
  const oidcReady =
    authMethod === "token" ||
    Boolean(
      oidcClientId.trim() &&
        oidcClientSecret.trim() &&
        oidcAudience.trim() &&
        oidcTokenEndpointUrl.trim(),
    );
  const canSubmit = useMemo(
    () =>
      Boolean(
        profileName.trim() &&
          serverAddr.trim() &&
          Number.isInteger(portNumber) &&
          portNumber > 0 &&
          portNumber <= 65535 &&
          oidcReady,
      ),
    [oidcReady, portNumber, profileName, serverAddr],
  );

  useEffect(() => {
    busyRef.current = busy;
  }, [busy]);

  useEffect(() => {
    onCloseRef.current = onClose;
  }, [onClose]);

  useEffect(() => {
    if (!open) return;
    if (!isEditing || !profile) {
      resetForm();
      return;
    }

    const authMethod = profile.authMethod === "oidc" ? "oidc" : "token";
    setProfileName(profile.displayName);
    setServerAddr(profile.serverAddr);
    setServerPort(String(profile.serverPort));
    setAuthMethod(authMethod);
    setAuthToken(profile.authToken ?? "");
    setOidcClientId(readTomlString(profile.rawToml, "auth.oidc.clientID"));
    setOidcClientSecret(
      readTomlString(profile.rawToml, "auth.oidc.clientSecret"),
    );
    setOidcAudience(readTomlString(profile.rawToml, "auth.oidc.audience"));
    setOidcTokenEndpointUrl(
      readTomlString(profile.rawToml, "auth.oidc.tokenEndpointURL"),
    );
  }, [isEditing, open, profile?.id]);

  useEffect(() => {
    if (!open) return;

    const previouslyFocused =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    nameInputRef.current?.focus();
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape" && !busyRef.current) onCloseRef.current();
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      previouslyFocused?.focus();
    };
  }, [open]);

  if (!open) return null;

  async function submit() {
    if (busy || !canSubmit) return;
    setBusy(true);
    try {
      await onSubmit({
        profileName: profileName.trim(),
        serverAddr: serverAddr.trim(),
        serverPort: portNumber,
        authMethod,
        authToken: authToken.trim() || null,
        oidcClientId: oidcClientId.trim() || null,
        oidcClientSecret: oidcClientSecret.trim() || null,
        oidcAudience: oidcAudience.trim() || null,
        oidcTokenEndpointUrl: oidcTokenEndpointUrl.trim() || null,
      });
      if (!isEditing) resetForm();
      onClose();
    } finally {
      setBusy(false);
    }
  }

  function resetForm() {
    setProfileName("");
    setServerAddr("");
    setServerPort("7000");
    setAuthMethod("token");
    setAuthToken("");
    setOidcClientId("");
    setOidcClientSecret("");
    setOidcAudience("");
    setOidcTokenEndpointUrl("");
  }

  return (
    <div className="modal-backdrop">
      <form
        className="modal profile-form-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="profile-editor-title"
        onSubmit={(event) => {
          event.preventDefault();
          void submit();
        }}
      >
        <h2 id="profile-editor-title">
          {isEditing ? "Edit frpc Profile" : "New frpc Profile"}
        </h2>
        {error ? <div className="error-banner">{error}</div> : null}

        <div className="form-grid">
          <label>
            Profile name
            <input
              ref={nameInputRef}
              value={profileName}
              disabled={busy}
              onChange={(event) => setProfileName(event.target.value)}
              autoComplete="off"
            />
          </label>
          <label>
            Server address
            <input
              value={serverAddr}
              disabled={busy}
              onChange={(event) => setServerAddr(event.target.value)}
              autoComplete="off"
            />
          </label>
          <label>
            Server port
            <input
              value={serverPort}
              disabled={busy}
              inputMode="numeric"
              onChange={(event) => setServerPort(event.target.value)}
            />
          </label>
          <label>
            Auth method
            <span className="select-wrapper">
              <select
                value={authMethod}
                disabled={busy}
                onChange={(event) =>
                  setAuthMethod(event.target.value as ProfileAuthMethod)
                }
              >
                <option value="token">token</option>
                <option value="oidc">oidc</option>
              </select>
            </span>
          </label>

          {authMethod === "token" ? (
            <label className="full-span">
              Auth token
              <input
                value={authToken}
                disabled={busy}
                onChange={(event) => setAuthToken(event.target.value)}
                autoComplete="off"
              />
            </label>
          ) : (
            <>
              <label>
                OIDC clientID
                <input
                  value={oidcClientId}
                  disabled={busy}
                  onChange={(event) => setOidcClientId(event.target.value)}
                  autoComplete="off"
                />
              </label>
              <label>
                OIDC clientSecret
                <input
                  type="password"
                  value={oidcClientSecret}
                  disabled={busy}
                  onChange={(event) => setOidcClientSecret(event.target.value)}
                  autoComplete="off"
                />
              </label>
              <label>
                OIDC audience
                <input
                  value={oidcAudience}
                  disabled={busy}
                  onChange={(event) => setOidcAudience(event.target.value)}
                  autoComplete="off"
                />
              </label>
              <label>
                OIDC tokenEndpointURL
                <input
                  value={oidcTokenEndpointUrl}
                  disabled={busy}
                  onChange={(event) =>
                    setOidcTokenEndpointUrl(event.target.value)
                  }
                  autoComplete="off"
                />
              </label>
            </>
          )}
        </div>

        <div className="modal-actions">
          <button
            type="button"
            className="command-button"
            disabled={busy}
            onClick={onClose}
          >
            Cancel
          </button>
          <button
            type="submit"
            className="command-button primary"
            disabled={busy || !canSubmit}
          >
            {busy ? (isEditing ? "Saving" : "Creating") : isEditing ? "Save" : "Create"}
          </button>
        </div>
      </form>
    </div>
  );
}

function readTomlString(rawToml: string, dottedPath: string): string {
  const direct = findDottedString(rawToml, dottedPath);
  if (direct !== null) return direct;

  const parts = dottedPath.split(".");
  const key = parts.pop();
  if (!key) return "";
  const section = parts.join(".");
  let inSection = false;
  for (const line of rawToml.split(/\r?\n/)) {
    const trimmed = line.trim();
    const sectionMatch = trimmed.match(/^\[([^\]]+)\]$/);
    if (sectionMatch) {
      inSection = sectionMatch[1] === section;
      continue;
    }
    if (!inSection) continue;
    const value = readTomlStringAssignment(trimmed, key);
    if (value !== null) return value;
  }

  return "";
}

function findDottedString(rawToml: string, dottedPath: string): string | null {
  for (const line of rawToml.split(/\r?\n/)) {
    const value = readTomlStringAssignment(line.trim(), dottedPath);
    if (value !== null) return value;
  }
  return null;
}

function readTomlStringAssignment(line: string, key: string): string | null {
  const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = line.match(new RegExp(`^${escapedKey}\\s*=\\s*\"((?:\\\\.|[^\"\\\\])*)\"`));
  if (!match) return null;
  return match[1].replace(/\\"/g, "\"").replace(/\\\\/g, "\\");
}

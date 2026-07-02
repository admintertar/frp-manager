import { useEffect, useMemo, useRef, useState } from "react";
import { sanitizePortInput } from "../lib/portInput";
import { proxyTypeOptions } from "../lib/proxyTypeOptions";
import type { AddProxyInput, ProxyConfig, ProxyType } from "../types";

interface Props {
  open: boolean;
  mode: "add" | "edit";
  serverAddr?: string;
  proxy?: ProxyConfig | null;
  error: string | null;
  onClose: () => void;
  onSubmit: (input: AddProxyInput) => Promise<void>;
}

export function ProxyEditor({
  open,
  mode,
  serverAddr,
  proxy,
  error,
  onClose,
  onSubmit,
}: Props) {
  const [name, setName] = useState("");
  const [proxyType, setProxyType] = useState<ProxyType>("http");
  const [localIp, setLocalIp] = useState("127.0.0.1");
  const [localPort, setLocalPort] = useState("");
  const [remotePort, setRemotePort] = useState("");
  const [subdomain, setSubdomain] = useState("");
  const [customDomains, setCustomDomains] = useState("");
  const [busy, setBusy] = useState(false);
  const nameInputRef = useRef<HTMLInputElement>(null);
  const busyRef = useRef(false);
  const onCloseRef = useRef(onClose);
  const isEditing = mode === "edit";

  const localPortNumber = Number(localPort);
  const remotePortNumber = Number(remotePort);
  const needsRemotePort = proxyType === "tcp" || proxyType === "udp";
  const showCustomDomains =
    isEditing && !needsRemotePort && (proxy?.customDomains.length ?? 0) > 0;
  const typeOptions = useMemo(
    () => proxyTypeOptions(mode, proxyType),
    [mode, proxyType],
  );
  const customDomainList = useMemo(
    () =>
      customDomains
        .split(/[\n,]/)
        .map((domain) => domain.trim())
        .filter(Boolean),
    [customDomains],
  );
  const mappingAddress = formatMappingAddress({
    customDomains: customDomainList,
    name,
    proxyType,
    remotePort,
    serverAddr,
    subdomain,
  });
  const canSubmit = useMemo(
    () =>
      Boolean(
        name.trim() &&
          Number.isInteger(localPortNumber) &&
          localPortNumber > 0 &&
          localPortNumber <= 65535 &&
          (!needsRemotePort ||
            (Number.isInteger(remotePortNumber) &&
              remotePortNumber > 0 &&
              remotePortNumber <= 65535)),
      ),
    [localPortNumber, name, needsRemotePort, remotePortNumber],
  );

  useEffect(() => {
    busyRef.current = busy;
  }, [busy]);

  useEffect(() => {
    onCloseRef.current = onClose;
  }, [onClose]);

  useEffect(() => {
    if (!open) return;
    if (!isEditing || !proxy) {
      resetForm();
      return;
    }

    setName(proxy.name);
    setProxyType(proxy.proxyType);
    setLocalIp(proxy.localIp ?? "127.0.0.1");
    setLocalPort(proxy.localPort ? String(proxy.localPort) : "");
    setRemotePort(proxy.remotePort ? String(proxy.remotePort) : "");
    setSubdomain(proxy.subdomain ?? "");
    setCustomDomains(proxy.customDomains.join(", "));
  }, [isEditing, open, proxy?.name]);

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
        name: name.trim(),
        proxyType,
        localIp: localIp.trim() || null,
        localPort: localPortNumber,
        remotePort: remotePort.trim() ? remotePortNumber : null,
        subdomain: subdomain.trim() || null,
        customDomains: showCustomDomains ? customDomainList : [],
      });
      if (!isEditing) resetForm();
      onClose();
    } finally {
      setBusy(false);
    }
  }

  function resetForm() {
    setName("");
    setProxyType("http");
    setLocalIp("127.0.0.1");
    setLocalPort("");
    setRemotePort("");
    setSubdomain("");
    setCustomDomains("");
  }

  return (
    <div className="modal-backdrop">
      <form
        className="modal proxy-form-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="proxy-editor-title"
        onSubmit={(event) => {
          event.preventDefault();
          void submit();
        }}
      >
        <h2 id="proxy-editor-title">
          {isEditing ? "Edit Proxy" : "Add Proxy"}
        </h2>
        {error ? <div className="error-banner">{error}</div> : null}

        <div className="form-grid">
          <label>
            Proxy name
            <input
              ref={nameInputRef}
              value={name}
              disabled={busy}
              onChange={(event) => setName(event.target.value)}
              autoComplete="off"
            />
          </label>
          <label>
            Mapping address
            <output className="mapping-preview">{mappingAddress}</output>
          </label>
          <label className="proxy-type-field">
            <span>Type</span>
            <span
              className="proxy-type-options"
              role="group"
              aria-label="Proxy type"
              style={{
                gridTemplateColumns: `repeat(${typeOptions.length}, minmax(0, 1fr))`,
              }}
            >
              {typeOptions.map((type) => (
                <button
                  className={
                    proxyType === type
                      ? "proxy-type-option selected"
                      : "proxy-type-option"
                  }
                  disabled={busy}
                  key={type}
                  type="button"
                  aria-pressed={proxyType === type}
                  onClick={() => setProxyType(type)}
                >
                  {type}
                </button>
              ))}
            </span>
          </label>
          <label>
            Local IP
            <input
              value={localIp}
              disabled={busy}
              onChange={(event) => setLocalIp(event.target.value)}
              autoComplete="off"
            />
          </label>
          <label>
            Local port
            <input
              value={localPort}
              disabled={busy}
              inputMode="numeric"
              pattern="[0-9]*"
              onChange={(event) =>
                setLocalPort(sanitizePortInput(event.target.value))
              }
            />
          </label>

          {needsRemotePort ? (
            <label>
              Remote port
              <input
                value={remotePort}
                disabled={busy}
                inputMode="numeric"
                pattern="[0-9]*"
                onChange={(event) =>
                  setRemotePort(sanitizePortInput(event.target.value))
                }
              />
            </label>
          ) : (
            <>
              <label>
                Subdomain
                <input
                  value={subdomain}
                  disabled={busy}
                  onChange={(event) => setSubdomain(event.target.value)}
                  autoComplete="off"
                />
              </label>
              {showCustomDomains ? (
                <label>
                  Custom domains
                  <input
                    value={customDomains}
                    disabled={busy}
                    onChange={(event) => setCustomDomains(event.target.value)}
                    autoComplete="off"
                  />
                </label>
              ) : null}
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
            {busy ? (isEditing ? "Saving" : "Adding") : isEditing ? "Save" : "Add Proxy"}
          </button>
        </div>
      </form>
    </div>
  );
}

function formatMappingAddress({
  customDomains,
  name,
  proxyType,
  remotePort,
  serverAddr,
  subdomain,
}: {
  customDomains: string[];
  name: string;
  proxyType: ProxyType;
  remotePort: string;
  serverAddr?: string;
  subdomain: string;
}): string {
  const host = serverAddr?.trim();
  if (!host) return "-";

  if (proxyType === "tcp" || proxyType === "udp") {
    const port = remotePort.trim();
    return port ? `${proxyType}://${host}:${port}` : `${proxyType}://${host}:...`;
  }

  const domain = customDomains[0] || buildSubdomainHost(subdomain, name, host);
  return domain ? `${proxyType}://${domain}` : `${proxyType}://...`;
}

function buildSubdomainHost(
  subdomain: string,
  name: string,
  serverAddr: string,
): string {
  const prefix = subdomain.trim() || name.trim();
  return prefix ? `${prefix}.${serverAddr}` : "";
}

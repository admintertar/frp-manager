import { AlertCircle, CheckCircle2, RefreshCw, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { checkRuntimeUpdate } from "../lib/api";
import type { RuntimeUpdateCheck } from "../types";

interface Props {
  open: boolean;
  onClose: () => void;
}

export function RuntimeSettings({ open, onClose }: Props) {
  const [result, setResult] = useState<RuntimeUpdateCheck | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const onCloseRef = useRef(onClose);

  useEffect(() => {
    onCloseRef.current = onClose;
  }, [onClose]);

  useEffect(() => {
    if (!open) return;

    const previouslyFocused =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    closeButtonRef.current?.focus();

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onCloseRef.current();
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      previouslyFocused?.focus();
    };
  }, [open]);

  if (!open) return null;

  async function check() {
    if (busy) return;
    setBusy(true);
    setError(null);
    try {
      setResult(await checkRuntimeUpdate());
    } catch (err) {
      setError(formatError(err));
    } finally {
      setBusy(false);
    }
  }

  const status = result
    ? result.updateAvailable
      ? "Update available"
      : "Runtime is current"
    : "Ready to check";

  return (
    <div className="modal-backdrop">
      <section
        className="modal runtime-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="runtime-settings-title"
      >
        <header className="modal-title-row">
          <div>
            <h2 id="runtime-settings-title">frpc Runtime</h2>
            <p className="muted">fatedier/frp GitHub Releases</p>
          </div>
          <button
            ref={closeButtonRef}
            type="button"
            className="icon-button"
            aria-label="Close runtime settings"
            onClick={onClose}
          >
            <X size={18} />
          </button>
        </header>

        {error ? <div className="error-banner">{error}</div> : null}

        <div
          className={`runtime-status ${
            result?.updateAvailable ? "runtime-status-update" : ""
          }`}
        >
          {result?.updateAvailable ? (
            <AlertCircle size={18} />
          ) : (
            <CheckCircle2 size={18} />
          )}
          <strong>{status}</strong>
        </div>

        <div className="runtime-result-grid">
          <div>
            <span>Current</span>
            <strong>{result?.currentVersion ?? "-"}</strong>
          </div>
          <div>
            <span>Latest</span>
            <strong>{result?.latestVersion ?? "-"}</strong>
          </div>
          <div className="runtime-asset">
            <span>Asset</span>
            <strong>{result?.assetName ?? "-"}</strong>
          </div>
        </div>

        <div className="modal-actions">
          <button type="button" className="command-button" onClick={onClose}>
            Close
          </button>
          <button
            type="button"
            className="command-button primary"
            disabled={busy}
            onClick={() => void check()}
          >
            <RefreshCw size={16} />
            {busy ? "Checking" : "Check Updates"}
          </button>
        </div>
      </section>
    </div>
  );
}

function formatError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "object" && err !== null && "message" in err) {
    const message = (err as { message?: unknown }).message;
    if (typeof message === "string") return message;
  }
  return String(err);
}

import { useEffect, useMemo, useState } from "react";
import type { MouseEvent } from "react";
import { Plus, Settings } from "lucide-react";
import type { ProfileSummary } from "../types";

interface Props {
  profiles: ProfileSummary[];
  selectedId?: string;
  runtimeVersion: string;
  onSelect: (profileId: string) => void;
  onEditProfile: (profileId: string) => void;
  onDeleteProfile: (profileId: string) => void;
  onImport: () => void;
  onOpenRuntimeSettings: () => void;
}

export function ProfileSidebar({
  profiles,
  selectedId,
  runtimeVersion,
  onSelect,
  onEditProfile,
  onDeleteProfile,
  onImport,
  onOpenRuntimeSettings,
}: Props) {
  const [menu, setMenu] = useState<{
    profileId: string;
    x: number;
    y: number;
  } | null>(null);
  const menuProfile = useMemo(
    () => profiles.find((profile) => profile.id === menu?.profileId),
    [menu?.profileId, profiles],
  );

  useEffect(() => {
    if (!menu) return;

    function closeMenu() {
      setMenu(null);
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") closeMenu();
    }

    window.addEventListener("click", closeMenu);
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("scroll", closeMenu, true);
    return () => {
      window.removeEventListener("click", closeMenu);
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("scroll", closeMenu, true);
    };
  }, [menu]);

  function openProfileMenu(
    event: MouseEvent<HTMLButtonElement>,
    profile: ProfileSummary,
  ) {
    event.preventDefault();
    onSelect(profile.id);
    setMenu({
      profileId: profile.id,
      x: Math.min(event.clientX, window.innerWidth - 168),
      y: Math.min(event.clientY, window.innerHeight - 88),
    });
  }

  function editMenuProfile() {
    if (!menuProfile) return;
    onEditProfile(menuProfile.id);
    setMenu(null);
  }

  function deleteMenuProfile() {
    if (!menuProfile) return;
    onDeleteProfile(menuProfile.id);
    setMenu(null);
  }

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <div>
          <strong>Profiles</strong>
          <span>{profiles.length} configured</span>
        </div>
        <button
          className="icon-button"
          aria-label="Add profile"
          onClick={onImport}
        >
          <Plus size={18} />
        </button>
      </div>
      <div className="profile-list">
        {profiles.map((profile) => (
          <button
            key={profile.id}
            className={
              profile.id === selectedId ? "profile-row selected" : "profile-row"
            }
            onClick={() => onSelect(profile.id)}
            onContextMenu={(event) => openProfileMenu(event, profile)}
            aria-current={profile.id === selectedId ? "true" : undefined}
          >
            <span className="profile-row-main">
              <strong>{profile.displayName}</strong>
              <em className={`state-pill state-${profile.runtimeState}`}>
                {profile.runtimeState}
              </em>
            </span>
            <span className="profile-row-meta">
              <small className="profile-row-endpoint">
                {profile.serverAddr}:{profile.serverPort}
              </small>
              <small className="profile-row-count">
                {formatProxyCount(profile.proxyCount)}
              </small>
            </span>
          </button>
        ))}
      </div>
      {menu && menuProfile ? (
        <div
          className="profile-context-menu"
          role="menu"
          aria-label={`${menuProfile.displayName} actions`}
          onClick={(event) => event.stopPropagation()}
          onContextMenu={(event) => event.preventDefault()}
          style={{ left: menu.x, top: menu.y }}
        >
          <button type="button" role="menuitem" onClick={editMenuProfile}>
            Edit
          </button>
          <button
            type="button"
            role="menuitem"
            className="danger-menu-item"
            onClick={deleteMenuProfile}
          >
            Delete
          </button>
        </div>
      ) : null}
      <div className="sidebar-runtime">
        <div>
          <span>frpc runtime</span>
          <strong>{runtimeVersion}</strong>
        </div>
        <button
          className="icon-button"
          aria-label="Runtime settings"
          onClick={onOpenRuntimeSettings}
        >
          <Settings size={17} />
        </button>
        <small>fatedier/frp GitHub Releases</small>
      </div>
    </aside>
  );
}

function formatProxyCount(count: number): string {
  return `${count} ${count === 1 ? "proxy" : "proxies"}`;
}

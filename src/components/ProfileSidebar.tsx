import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import type { MouseEvent } from "react";
import { Languages, Plus, Settings } from "lucide-react";
import {
  localeLabel,
  otherLocale,
  useLocale,
  useTranslation,
} from "../lib/i18n";
import type { Locale } from "../lib/i18n";
import type { ProfileSummary } from "../types";

interface Props {
  profiles: ProfileSummary[];
  selectedId?: string;
  runtimeVersion: string;
  onSelect: (profileId: string) => void;
  onEditProfile: (profileId: string) => void;
  onDeleteProfile: (profileId: string) => void;
  onToggleAutoStart: (profileId: string, autoStart: boolean) => void;
  onChangeLocale: (locale: Locale) => void;
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
  onToggleAutoStart,
  onChangeLocale,
  onImport,
  onOpenRuntimeSettings,
}: Props) {
  const t = useTranslation();
  const locale = useLocale();
  const nextLocale = otherLocale(locale);
  const [menu, setMenu] = useState<{
    profileId: string;
    x: number;
    y: number;
  } | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);
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

  // The menu is sized by its labels, so its real size is only known once it is
  // in the DOM. Clamp it into the viewport here rather than guessing a size at
  // open time; the layout effect runs before paint, so nothing visibly jumps.
  useLayoutEffect(() => {
    const element = menuRef.current;
    if (!menu || !element) return;

    const margin = 8;
    const { width, height } = element.getBoundingClientRect();
    const x = Math.max(
      margin,
      Math.min(menu.x, window.innerWidth - width - margin),
    );
    const y = Math.max(
      margin,
      Math.min(menu.y, window.innerHeight - height - margin),
    );

    if (x !== menu.x || y !== menu.y) {
      setMenu({ ...menu, x, y });
    }
  }, [menu]);

  function openProfileMenu(
    event: MouseEvent<HTMLButtonElement>,
    profile: ProfileSummary,
  ) {
    event.preventDefault();
    onSelect(profile.id);
    setMenu({ profileId: profile.id, x: event.clientX, y: event.clientY });
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

  function toggleMenuAutoStart() {
    if (!menuProfile) return;
    onToggleAutoStart(menuProfile.id, !menuProfile.autoStart);
    setMenu(null);
  }

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <div>
          <strong>{t("sidebar.title")}</strong>
          <span>{t("sidebar.configured", { count: profiles.length })}</span>
        </div>
        <span className="sidebar-header-actions">
          <button
            className="icon-button"
            aria-label={t("sidebar.switchLanguage", {
              language: localeLabel(nextLocale),
            })}
            title={localeLabel(nextLocale)}
            onClick={() => onChangeLocale(nextLocale)}
          >
            <Languages size={17} />
          </button>
          <button
            className="icon-button"
            aria-label={t("sidebar.addProfile")}
            onClick={onImport}
          >
            <Plus size={18} />
          </button>
        </span>
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
                {t(`state.${profile.runtimeState}`)}
              </em>
            </span>
            <span className="profile-row-meta">
              <small className="profile-row-endpoint">
                {profile.serverAddr}:{profile.serverPort}
              </small>
              {profile.autoStart ? (
                <small
                  className="profile-row-auto"
                  title={t("sidebar.autoBadgeTitle")}
                >
                  {t("sidebar.autoBadge")}
                </small>
              ) : null}
              <small className="profile-row-count">
                {profile.proxyCount === 1
                  ? t("sidebar.proxyCountOne", { count: profile.proxyCount })
                  : t("sidebar.proxyCountOther", { count: profile.proxyCount })}
              </small>
            </span>
          </button>
        ))}
      </div>
      {menu && menuProfile ? (
        <div
          className="profile-context-menu"
          ref={menuRef}
          role="menu"
          aria-label={t("sidebar.menuLabel", {
            name: menuProfile.displayName,
          })}
          onClick={(event) => event.stopPropagation()}
          onContextMenu={(event) => event.preventDefault()}
          style={{ left: menu.x, top: menu.y }}
        >
          <button type="button" role="menuitem" onClick={editMenuProfile}>
            {t("common.edit")}
          </button>
          <button
            type="button"
            role="menuitemcheckbox"
            aria-checked={menuProfile.autoStart}
            onClick={toggleMenuAutoStart}
          >
            <span>{t("sidebar.startOnLaunch")}</span>
            <span className="menu-state">
              {menuProfile.autoStart
                ? t("sidebar.autoStartOn")
                : t("sidebar.autoStartOff")}
            </span>
          </button>
          <button
            type="button"
            role="menuitem"
            className="danger-menu-item"
            onClick={deleteMenuProfile}
          >
            {t("common.delete")}
          </button>
        </div>
      ) : null}
      <div className="sidebar-runtime">
        <div>
          <span>{t("sidebar.runtime")}</span>
          <strong>{runtimeVersion}</strong>
        </div>
        <button
          className="icon-button"
          aria-label={t("sidebar.runtimeSettings")}
          onClick={onOpenRuntimeSettings}
        >
          <Settings size={17} />
        </button>
        <small>fatedier/frp GitHub Releases</small>
      </div>
    </aside>
  );
}

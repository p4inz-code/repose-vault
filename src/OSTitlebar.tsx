import React, { useState } from 'react';
import type { OSPlatform, ThemeMode } from './types';
import { ReposeLogo } from './ReposeLogo';
import { Shield, Lock, Search, Plus, Sun, Moon } from 'lucide-react';

interface OSTitlebarProps {
  os: OSPlatform;
  onSelectOS: (os: OSPlatform) => void;
  theme: ThemeMode;
  onToggleTheme: () => void;
  isLocked: boolean;
  onLock?: () => void;
  onCommandOpen?: () => void;
  onCreateOpen?: () => void;
  activeTab?: 'vault' | 'settings';
  onSelectTab?: (tab: 'vault' | 'settings') => void;
}

export function OSTitlebar({
  os,
  onSelectOS,
  theme,
  onToggleTheme,
  isLocked,
  onLock,
  onCommandOpen,
  onCreateOpen,
  activeTab = 'vault',
  onSelectTab,
}: OSTitlebarProps) {
  const isDark = theme === 'dark';
  const [showSnapLayouts, setShowSnapLayouts] = useState<boolean>(false);
  const [trafficHover, setTrafficHover] = useState<boolean>(false);

  const handleMinimize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().minimize();
    } catch {
      // Browser preview no-op
    }
  };

  const handleToggleMaximize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().toggleMaximize();
    } catch {
      // Browser preview no-op
    }
  };

  const handleClose = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().close();
    } catch {
      // Browser preview no-op
    }
  };

  return (
    <header
      className={`h-11 px-3 flex items-center justify-between shrink-0 select-none border-b transition-colors ${
        isDark
          ? 'bg-[#0b0c10]/95 border-zinc-800/80 text-zinc-300'
          : 'bg-[#f4f5f7]/95 border-zinc-200/90 text-zinc-700'
      }`}
      style={{
        WebkitAppRegion: 'drag', // Native Tauri window drag region
      } as React.CSSProperties}
    >
      {/* LEFT SECTION */}
      <div className="flex items-center gap-3">
        {/* macOS Traffic Lights */}
        {os === 'macos' && (
          <div
            className="flex items-center gap-2 pl-1 pr-2 py-1"
            onMouseEnter={() => setTrafficHover(true)}
            onMouseLeave={() => setTrafficHover(false)}
            style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
          >
            {/* Close */}
            <button
              onClick={handleClose}
              className="w-3 h-3 rounded-full bg-[#FF5F56] border border-[#E0443E] flex items-center justify-center cursor-pointer transition-transform active:scale-90"
              title="Close (Cmd+W)"
            >
              {trafficHover && (
                <svg className="w-1.5 h-1.5 text-[#4D0000]" viewBox="0 0 6 6" fill="currentColor">
                  <path d="M1 1l4 4m0-4L1 5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
                </svg>
              )}
            </button>
            {/* Minimize */}
            <button
              onClick={handleMinimize}
              className="w-3 h-3 rounded-full bg-[#FFBD2E] border border-[#DEA123] flex items-center justify-center cursor-pointer transition-transform active:scale-90"
              title="Minimize (Cmd+M)"
            >
              {trafficHover && (
                <svg className="w-1.5 h-1.5 text-[#5B3C00]" viewBox="0 0 6 6" fill="currentColor">
                  <path d="M1 3h4" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
                </svg>
              )}
            </button>
            {/* Zoom / Fullscreen */}
            <button
              onClick={handleToggleMaximize}
              className="w-3 h-3 rounded-full bg-[#27C93F] border border-[#1AAB29] flex items-center justify-center cursor-pointer transition-transform active:scale-90"
              title="Fullscreen"
            >
              {trafficHover && (
                <svg className="w-1.5 h-1.5 text-[#004A00]" viewBox="0 0 6 6" fill="currentColor">
                  <path d="M1 5l4-4M1 1h3M5 5V2" stroke="currentColor" strokeWidth="1.0" strokeLinecap="round" />
                </svg>
              )}
            </button>
          </div>
        )}

        {/* Brand & App Name */}
        <div className="flex items-center gap-2" style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}>
          <ReposeLogo size={18} />
          <span className={`font-semibold text-xs tracking-tight ${isDark ? 'text-zinc-100' : 'text-zinc-900'}`}>
            Repose
          </span>
          <span
            className={`text-[9px] px-1.5 py-0.5 rounded font-mono font-medium border ${
              isDark
                ? 'bg-zinc-900/80 border-zinc-800 text-zinc-400'
                : 'bg-zinc-200/70 border-zinc-300 text-zinc-600'
            }`}
          >
            v1.0.0
          </span>
        </div>

        {/* Tab Switcher (when unlocked) */}
        {!isLocked && onSelectTab && (
          <div
            className="flex items-center gap-1 ml-2 text-xs"
            style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
          >
            <button
              onClick={() => onSelectTab('vault')}
              className={`px-2.5 py-0.5 rounded cursor-pointer font-medium transition-colors ${
                activeTab === 'vault'
                  ? isDark
                    ? 'bg-zinc-800 text-white'
                    : 'bg-zinc-200 text-black'
                  : 'opacity-60 hover:opacity-100'
              }`}
            >
              Space Vault
            </button>
            <button
              onClick={() => onSelectTab('settings')}
              className={`px-2.5 py-0.5 rounded cursor-pointer font-medium transition-colors ${
                activeTab === 'settings'
                  ? isDark
                    ? 'bg-zinc-800 text-white'
                    : 'bg-zinc-200 text-black'
                  : 'opacity-60 hover:opacity-100'
              }`}
            >
              Settings
            </button>
          </div>
        )}
      </div>

      {/* CENTER: OS Switcher & Security Invariant */}
      <div
        className="flex items-center gap-2"
        style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
      >
        {/* Platform View Switcher */}
        <div
          className={`flex items-center p-0.5 rounded-lg border text-[11px] font-medium transition-colors ${
            isDark ? 'bg-zinc-900/90 border-zinc-800' : 'bg-zinc-200/80 border-zinc-300'
          }`}
          title="Switch Operating System Frame Simulation"
        >
          <button
            onClick={() => onSelectOS('windows')}
            className={`px-2 py-0.5 rounded cursor-pointer transition-colors ${
              os === 'windows'
                ? isDark
                  ? 'bg-zinc-800 text-white font-semibold shadow-xs'
                  : 'bg-white text-zinc-900 font-semibold shadow-xs'
                : 'opacity-60 hover:opacity-100'
            }`}
          >
            Windows 11
          </button>
          <button
            onClick={() => onSelectOS('macos')}
            className={`px-2 py-0.5 rounded cursor-pointer transition-colors ${
              os === 'macos'
                ? isDark
                  ? 'bg-zinc-800 text-white font-semibold shadow-xs'
                  : 'bg-white text-zinc-900 font-semibold shadow-xs'
                : 'opacity-60 hover:opacity-100'
            }`}
          >
            macOS
          </button>
          <button
            onClick={() => onSelectOS('linux')}
            className={`px-2 py-0.5 rounded cursor-pointer transition-colors ${
              os === 'linux'
                ? isDark
                  ? 'bg-zinc-800 text-white font-semibold shadow-xs'
                  : 'bg-white text-zinc-900 font-semibold shadow-xs'
                : 'opacity-60 hover:opacity-100'
            }`}
          >
            Linux CSD
          </button>
        </div>

        {/* Security Badge */}
        <div
          className={`hidden md:flex items-center gap-1.5 px-2 py-0.5 rounded text-[10px] font-mono border ${
            isDark
              ? 'bg-emerald-950/20 border-emerald-900/50 text-emerald-400'
              : 'bg-emerald-50 border-emerald-200 text-emerald-700'
          }`}
          title={
            os === 'windows'
              ? 'Win32 SetWindowDisplayAffinity(WDA_MONITOR) active: Blocks OBS & screen recorders'
              : os === 'macos'
              ? 'macOS LocalAuthentication & Hardened Runtime active'
              : 'Linux mlock() & Wayland Secure Surface active'
          }
        >
          <Shield className="w-2.5 h-2.5" />
          <span>
            {os === 'windows'
              ? 'WDA_MONITOR'
              : os === 'macos'
              ? 'Hardened'
              : 'mlock()'}
          </span>
        </div>
      </div>

      {/* RIGHT SECTION */}
      <div
        className="flex items-center gap-2"
        style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
      >
        {/* Action buttons (when unlocked) */}
        {!isLocked && (
          <>
            {onCommandOpen && (
              <button
                onClick={onCommandOpen}
                className={`flex items-center gap-1 border px-2 py-0.5 rounded text-xs cursor-pointer ${
                  isDark
                    ? 'bg-zinc-900 border-zinc-800 text-zinc-400 hover:text-white'
                    : 'bg-zinc-100 border-zinc-200 text-zinc-600 hover:text-black'
                }`}
                title={`Quick Jump (${os === 'macos' ? 'Cmd+K' : 'Ctrl+K'})`}
              >
                <Search className="w-3 h-3" />
                <kbd
                  className={`text-[9px] px-1 rounded font-mono ${
                    isDark ? 'bg-zinc-800 text-zinc-400' : 'bg-zinc-200 text-zinc-600'
                  }`}
                >
                  {os === 'macos' ? '⌘K' : 'Ctrl+K'}
                </kbd>
              </button>
            )}

            {onCreateOpen && (
              <button
                onClick={onCreateOpen}
                className={`flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium cursor-pointer ${
                  isDark
                    ? 'bg-white hover:bg-zinc-200 text-black'
                    : 'bg-black hover:bg-zinc-800 text-white'
                }`}
                title={`New Record (${os === 'macos' ? 'Cmd+N' : 'Ctrl+N'})`}
              >
                <Plus className="w-3 h-3" />
                <span className="hidden sm:inline">New</span>
              </button>
            )}

            {onLock && (
              <button
                onClick={onLock}
                className={`flex items-center gap-1 px-2 py-0.5 rounded border cursor-pointer text-xs ${
                  isDark
                    ? 'border-zinc-800 text-zinc-400 hover:text-rose-400'
                    : 'border-zinc-200 text-zinc-600 hover:text-rose-600'
                }`}
                title={`Lock Vault (${os === 'macos' ? 'Cmd+L' : 'Ctrl+L'})`}
              >
                <Lock className="w-3 h-3" />
                <span className="hidden sm:inline">Lock</span>
              </button>
            )}
          </>
        )}

        {/* Theme Toggle */}
        <button
          onClick={onToggleTheme}
          className={`p-1 rounded border cursor-pointer ${
            isDark
              ? 'border-zinc-800 text-zinc-400 hover:text-white'
              : 'border-zinc-200 text-zinc-600 hover:text-black'
          }`}
          title="Toggle Dark / Light Theme"
        >
          {isDark ? <Sun className="w-3 h-3" /> : <Moon className="w-3 h-3" />}
        </button>

        {/* WINDOWS 11 CAPTION BUTTONS */}
        {os === 'windows' && (
          <div className="flex items-center ml-1 border-l pl-2 border-zinc-800/50">
            {/* Minimize */}
            <button
              onClick={handleMinimize}
              className="w-7 h-7 flex items-center justify-center hover:bg-zinc-800/60 rounded cursor-pointer transition-colors"
              title="Minimize"
            >
              <svg className="w-3 h-3" viewBox="0 0 10 1" fill="currentColor">
                <rect width="10" height="1" />
              </svg>
            </button>

            {/* Maximize / Snap Layouts */}
            <div className="relative">
              <button
                onClick={handleToggleMaximize}
                onMouseEnter={() => setShowSnapLayouts(true)}
                onMouseLeave={() => setShowSnapLayouts(false)}
                className="w-7 h-7 flex items-center justify-center hover:bg-zinc-800/60 rounded cursor-pointer transition-colors"
                title="Maximize (Hover for Windows 11 Snap Layouts)"
              >
                <svg className="w-2.5 h-2.5" viewBox="0 0 10 10" fill="none" stroke="currentColor">
                  <rect x="0.5" y="0.5" width="9" height="9" strokeWidth="1" />
                </svg>
              </button>

              {/* Windows 11 Snap Layouts Flyout Preview */}
              {showSnapLayouts && (
                <div
                  className={`absolute right-0 top-8 z-50 p-2 rounded-lg border shadow-2xl flex gap-2 ${
                    isDark ? 'bg-zinc-900 border-zinc-700' : 'bg-white border-zinc-300'
                  }`}
                  style={{ width: '190px' }}
                >
                  {/* Layout 1: 50 / 50 */}
                  <div className="flex-1 flex gap-1 h-12 p-1 rounded border border-zinc-700 hover:border-blue-500 cursor-pointer bg-zinc-800/40">
                    <div className="w-1/2 bg-blue-500/20 hover:bg-blue-500/40 rounded-xs border border-blue-500/30" />
                    <div className="w-1/2 bg-zinc-700/20 hover:bg-blue-500/40 rounded-xs border border-zinc-700/30" />
                  </div>
                  {/* Layout 2: 60 / 40 */}
                  <div className="flex-1 flex gap-1 h-12 p-1 rounded border border-zinc-700 hover:border-blue-500 cursor-pointer bg-zinc-800/40">
                    <div className="w-2/3 bg-blue-500/20 hover:bg-blue-500/40 rounded-xs border border-blue-500/30" />
                    <div className="w-1/3 bg-zinc-700/20 hover:bg-blue-500/40 rounded-xs border border-zinc-700/30" />
                  </div>
                </div>
              )}
            </div>

            {/* Close */}
            <button
              onClick={handleClose}
              className="w-7 h-7 flex items-center justify-center hover:bg-rose-600 hover:text-white rounded cursor-pointer transition-colors"
              title="Close (Locks Vault & Minimizes to Tray)"
            >
              <svg className="w-2.5 h-2.5" viewBox="0 0 10 10" fill="none" stroke="currentColor">
                <path d="M1 1L9 9M9 1L1 9" strokeWidth="1.2" strokeLinecap="round" />
              </svg>
            </button>
          </div>
        )}

        {/* LINUX CSD WINDOW CONTROLS */}
        {os === 'linux' && (
          <div className="flex items-center gap-1 ml-1 border-l pl-2 border-zinc-800/50">
            <button
              className="w-6 h-6 rounded-full flex items-center justify-center hover:bg-zinc-800/70 text-zinc-400 hover:text-white cursor-pointer"
              title="Minimize"
            >
              <svg className="w-2.5 h-2.5" viewBox="0 0 10 1" fill="currentColor">
                <rect width="10" height="1" />
              </svg>
            </button>
            <button
              className="w-6 h-6 rounded-full flex items-center justify-center hover:bg-zinc-800/70 text-zinc-400 hover:text-white cursor-pointer"
              title="Maximize"
            >
              <svg className="w-2.5 h-2.5" viewBox="0 0 10 10" fill="none" stroke="currentColor">
                <rect x="0.5" y="0.5" width="9" height="9" strokeWidth="1" />
              </svg>
            </button>
            <button
              className="w-6 h-6 rounded-full flex items-center justify-center bg-zinc-800/40 hover:bg-rose-600 hover:text-white text-zinc-300 cursor-pointer"
              title="Close"
            >
              <svg className="w-2 h-2" viewBox="0 0 8 8" fill="none" stroke="currentColor">
                <path d="M1 1L7 7M7 1L1 7" strokeWidth="1.2" strokeLinecap="round" />
              </svg>
            </button>
          </div>
        )}
      </div>
    </header>
  );
}

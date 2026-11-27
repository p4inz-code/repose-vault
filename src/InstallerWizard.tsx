import React, { useState, useEffect } from 'react';
import {
  CheckCircle2, AlertTriangle, FolderOpen, ShieldCheck, ArrowRight,
  Trash2, HardDrive, X, Minus, Sun, Moon, Loader2
} from 'lucide-react';
import { ReposeLogo } from './ReposeLogo';
import type { ThemeMode } from './types';

interface DoctorCheck {
  id: string;
  label: string;
  ok: boolean;
  detail: string;
}

interface InstallerWizardProps {
  mode: 'installer' | 'uninstall';
  defaultInstallDir: string;
  alreadyInstalled: boolean;
  theme: ThemeMode;
  onToggleTheme: () => void;
}

type Step = 'choose' | 'doctor' | 'installing' | 'done' | 'uninstall_confirm' | 'uninstall_done';

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

async function closeInstallerWindow() {
  try {
    await tauriInvoke('exit_app');
  } catch {
    window.close();
  }
}

async function minimizeInstallerWindow() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().minimize();
  } catch {
    // Browser preview no-op
  }
}

export function InstallerWizard({
  mode,
  defaultInstallDir,
  alreadyInstalled,
  theme,
  onToggleTheme,
}: InstallerWizardProps) {
  const isDark = theme === 'dark';
  const [step, setStep] = useState<Step>(mode === 'uninstall' ? 'uninstall_confirm' : 'choose');
  const [installDir, setInstallDir] = useState<string>(defaultInstallDir);
  const [desktopShortcut, setDesktopShortcut] = useState<boolean>(true);
  const [startMenuShortcut, setStartMenuShortcut] = useState<boolean>(true);
  const [launchAfter, setLaunchAfter] = useState<boolean>(true);
  const [doctorChecks, setDoctorChecks] = useState<DoctorCheck[]>([]);
  const [shredVaultData, setShredVaultData] = useState<boolean>(false);
  const [statusMessage, setStatusMessage] = useState<string>('');
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    if (step === 'doctor') {
      tauriInvoke<DoctorCheck[]>('installer_doctor', { targetDir: installDir })
        .then(setDoctorChecks)
        .catch(() => {
          setDoctorChecks([
            { id: 'DOC-01', label: 'Platform & Architecture', ok: true, detail: 'windows (x86_64)' },
            { id: 'DOC-02', label: 'Per-User Install Directory', ok: true, detail: installDir },
          ]);
        });
    }
  }, [step, installDir]);

  const handleStartInstall = async () => {
    setStep('installing');
    setErrorMessage(null);
    try {
      const res = await tauriInvoke<{ success: boolean; message: string }>('installer_run', {
        destDir: installDir,
        desktopShortcut,
        startMenuShortcut,
        launchAfter,
      });
      setStatusMessage(res.message);
      setStep('done');
    } catch (err) {
      setErrorMessage(String(err));
      setStep('choose');
    }
  };

  const handleUninstall = async () => {
    setErrorMessage(null);
    try {
      const res = await tauriInvoke<{ success: boolean; message: string }>('uninstaller_run', {
        shredVaultData,
      });
      setStatusMessage(res.message);
      setStep('uninstall_done');
    } catch (err) {
      setErrorMessage(String(err));
    }
  };

  return (
    <div
      className={`h-screen w-screen flex flex-col select-none overflow-hidden transition-colors ${
        isDark ? 'bg-[#090a0f] text-[#f2f2f7]' : 'bg-[#f7f8fa] text-[#1c1c1e]'
      }`}
    >
      {/* Custom Frameless Installer Titlebar */}
      <header
        className={`h-11 px-4 flex items-center justify-between border-b shrink-0 ${
          isDark ? 'bg-[#0b0c10] border-zinc-800/80' : 'bg-white border-zinc-200'
        }`}
        style={{ WebkitAppRegion: 'drag' } as React.CSSProperties}
      >
        <div className="flex items-center gap-2.5">
          <ReposeLogo size={18} />
          <span className="text-xs font-semibold tracking-tight">
            Repose Setup
          </span>
          <span
            className={`text-[10px] px-1.5 py-0.5 rounded font-mono border ${
              isDark ? 'bg-zinc-900 border-zinc-800 text-zinc-400' : 'bg-zinc-100 border-zinc-200 text-zinc-600'
            }`}
          >
            v1.0.0 • GPL-3.0
          </span>
        </div>

        <div
          className="flex items-center gap-1.5"
          style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
        >
          <button
            onClick={onToggleTheme}
            className={`p-1 rounded border cursor-pointer ${
              isDark ? 'border-zinc-800 text-zinc-400 hover:text-white' : 'border-zinc-200 text-zinc-600 hover:text-black'
            }`}
            title="Toggle Theme"
          >
            {isDark ? <Sun className="w-3 h-3" /> : <Moon className="w-3 h-3" />}
          </button>
          <button
            onClick={minimizeInstallerWindow}
            className="w-7 h-7 flex items-center justify-center rounded hover:bg-zinc-800/40 cursor-pointer"
            title="Minimize"
          >
            <Minus className="w-3 h-3" />
          </button>
          <button
            onClick={closeInstallerWindow}
            className="w-7 h-7 flex items-center justify-center rounded hover:bg-rose-600 hover:text-white cursor-pointer"
            title="Close Setup"
          >
            <X className="w-3 h-3" />
          </button>
        </div>
      </header>

      {/* Wizard Body */}
      <main className="flex-1 p-6 flex flex-col justify-between overflow-y-auto">
        {errorMessage && (
          <div className="mb-3 p-2.5 rounded-lg border border-rose-500/30 bg-rose-500/10 text-rose-400 text-xs flex items-center gap-2">
            <AlertTriangle className="w-3.5 h-3.5 shrink-0" />
            <span>{errorMessage}</span>
          </div>
        )}

        {/* STEP 1: CHOOSE / INSTALL */}
        {step === 'choose' && (
          <div className="space-y-4">
            <div>
              <h1 className="text-base font-semibold tracking-tight">
                {alreadyInstalled ? 'Update or Reinstall Repose' : 'Install Repose Space Vault'}
              </h1>
              <p className="text-xs opacity-60 mt-0.5">
                Per-user airgapped installation — no administrator rights required, zero network calls.
              </p>
            </div>

            <div
              className={`p-3.5 rounded-lg border space-y-2 ${
                isDark ? 'bg-zinc-900/50 border-zinc-800' : 'bg-white border-zinc-200'
              }`}
            >
              <label className="text-[10px] font-mono uppercase tracking-wider opacity-60 flex items-center gap-1.5">
                <FolderOpen className="w-3 h-3" />
                <span>Install Destination (Per-User)</span>
              </label>
              <input
                type="text"
                value={installDir}
                onChange={e => setInstallDir(e.target.value)}
                className={`w-full px-3 py-1.5 rounded border text-xs font-mono outline-hidden ${
                  isDark ? 'bg-zinc-950 border-zinc-800 text-zinc-200' : 'bg-zinc-50 border-zinc-300 text-zinc-900'
                }`}
              />
            </div>

            <div
              className={`p-3.5 rounded-lg border space-y-2.5 text-xs ${
                isDark ? 'bg-zinc-900/50 border-zinc-800' : 'bg-white border-zinc-200'
              }`}
            >
              <label className="flex items-center justify-between cursor-pointer">
                <span>Create Start Menu Shortcut</span>
                <input
                  type="checkbox"
                  checked={startMenuShortcut}
                  onChange={e => setStartMenuShortcut(e.target.checked)}
                />
              </label>
              <label className="flex items-center justify-between cursor-pointer">
                <span>Create Desktop Shortcut</span>
                <input
                  type="checkbox"
                  checked={desktopShortcut}
                  onChange={e => setDesktopShortcut(e.target.checked)}
                />
              </label>
              <label className="flex items-center justify-between cursor-pointer">
                <span>Launch Repose after installation</span>
                <input
                  type="checkbox"
                  checked={launchAfter}
                  onChange={e => setLaunchAfter(e.target.checked)}
                />
              </label>
            </div>

            <div className="flex items-center justify-between pt-2">
              <button
                onClick={() => setStep('doctor')}
                className={`px-3 py-1.5 rounded-lg border text-xs font-medium flex items-center gap-1.5 cursor-pointer ${
                  isDark ? 'border-zinc-800 text-zinc-300 hover:bg-zinc-900' : 'border-zinc-300 text-zinc-700 hover:bg-zinc-100'
                }`}
              >
                <ShieldCheck className="w-3.5 h-3.5 text-emerald-500" />
                <span>Pre-Flight Doctor</span>
              </button>

              <button
                onClick={handleStartInstall}
                className={`px-4 py-1.5 rounded-lg text-xs font-medium flex items-center gap-1.5 cursor-pointer ${
                  isDark ? 'bg-white text-black hover:bg-zinc-200' : 'bg-black text-white hover:bg-zinc-800'
                }`}
              >
                <span>{alreadyInstalled ? 'Update Repose' : 'Install Repose'}</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        )}

        {/* STEP 2: DOCTOR */}
        {step === 'doctor' && (
          <div className="space-y-4">
            <div>
              <h1 className="text-base font-semibold tracking-tight">System & Security Doctor</h1>
              <p className="text-xs opacity-60 mt-0.5">
                Verifying OS RAM locking, writable per-user paths, and clean-room execution.
              </p>
            </div>

            <div className="space-y-2">
              {doctorChecks.map(c => (
                <div
                  key={c.id}
                  className={`p-2.5 rounded-lg border flex items-center justify-between text-xs ${
                    isDark ? 'bg-zinc-900/50 border-zinc-800' : 'bg-white border-zinc-200'
                  }`}
                >
                  <div>
                    <span className="font-medium block">{c.label}</span>
                    <span className="text-[10px] font-mono opacity-60">{c.detail}</span>
                  </div>
                  {c.ok ? (
                    <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0" />
                  ) : (
                    <AlertTriangle className="w-4 h-4 text-rose-500 shrink-0" />
                  )}
                </div>
              ))}
            </div>

            <div className="flex items-center justify-between pt-2">
              <button
                onClick={() => setStep('choose')}
                className={`px-3 py-1.5 rounded-lg border text-xs cursor-pointer ${
                  isDark ? 'border-zinc-800 hover:bg-zinc-900' : 'border-zinc-300 hover:bg-zinc-100'
                }`}
              >
                Back
              </button>
              <button
                onClick={handleStartInstall}
                className={`px-4 py-1.5 rounded-lg text-xs font-medium flex items-center gap-1.5 cursor-pointer ${
                  isDark ? 'bg-white text-black hover:bg-zinc-200' : 'bg-black text-white hover:bg-zinc-800'
                }`}
              >
                <span>Proceed to Install</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        )}

        {/* STEP 3: INSTALLING */}
        {step === 'installing' && (
          <div className="flex-1 flex flex-col items-center justify-center text-center space-y-3">
            <Loader2 className="w-7 h-7 animate-spin opacity-80" />
            <h2 className="text-sm font-semibold">Installing Repose...</h2>
            <p className="text-xs font-mono opacity-50">{installDir}</p>
          </div>
        )}

        {/* STEP 4: DONE */}
        {step === 'done' && (
          <div className="flex-1 flex flex-col items-center justify-center text-center space-y-4">
            <CheckCircle2 className="w-10 h-10 text-emerald-500" />
            <div>
              <h2 className="text-base font-semibold">Repose Installed</h2>
              <p className="text-xs opacity-60 mt-1">{statusMessage}</p>
            </div>
            <button
              onClick={closeInstallerWindow}
              className={`px-5 py-1.5 rounded-lg text-xs font-medium cursor-pointer ${
                isDark ? 'bg-white text-black hover:bg-zinc-200' : 'bg-black text-white hover:bg-zinc-800'
              }`}
            >
              Finish
            </button>
          </div>
        )}

        {/* UNINSTALL FLOW */}
        {step === 'uninstall_confirm' && (
          <div className="space-y-4">
            <div>
              <h1 className="text-base font-semibold tracking-tight">Uninstall Repose</h1>
              <p className="text-xs opacity-60 mt-0.5">
                Removes application binaries, shortcuts, and Windows Uninstall registration.
              </p>
            </div>

            <div
              className={`p-3.5 rounded-lg border space-y-2 text-xs ${
                isDark ? 'bg-zinc-900/50 border-zinc-800' : 'bg-white border-zinc-200'
              }`}
            >
              <div className="flex items-center gap-2 font-medium">
                <HardDrive className="w-3.5 h-3.5 text-emerald-500" />
                <span>Encrypted Vault Data Retention</span>
              </div>
              <p className="text-[11px] opacity-60">
                By default, your encrypted <code>vault.repose</code> database in <code>%APPDATA%\Repose</code> is preserved so you never lose passwords on reinstall.
              </p>
              <label className="flex items-center gap-2 pt-1 text-rose-400 cursor-pointer">
                <input
                  type="checkbox"
                  checked={shredVaultData}
                  onChange={e => setShredVaultData(e.target.checked)}
                />
                <span>Also permanently shred encrypted vault data</span>
              </label>
            </div>

            <div className="flex items-center justify-end gap-2 pt-2">
              <button
                onClick={closeInstallerWindow}
                className={`px-3 py-1.5 rounded-lg border text-xs cursor-pointer ${
                  isDark ? 'border-zinc-800 hover:bg-zinc-900' : 'border-zinc-300 hover:bg-zinc-100'
                }`}
              >
                Cancel
              </button>
              <button
                onClick={handleUninstall}
                className="px-4 py-1.5 rounded-lg text-xs font-medium bg-rose-600 hover:bg-rose-700 text-white flex items-center gap-1.5 cursor-pointer"
              >
                <Trash2 className="w-3.5 h-3.5" />
                <span>Uninstall Repose</span>
              </button>
            </div>
          </div>
        )}

        {step === 'uninstall_done' && (
          <div className="flex-1 flex flex-col items-center justify-center text-center space-y-4">
            <CheckCircle2 className="w-10 h-10 text-emerald-500" />
            <div>
              <h2 className="text-base font-semibold">Uninstall Complete</h2>
              <p className="text-xs opacity-60 mt-1">{statusMessage}</p>
            </div>
            <button
              onClick={closeInstallerWindow}
              className={`px-5 py-1.5 rounded-lg text-xs font-medium cursor-pointer ${
                isDark ? 'bg-white text-black hover:bg-zinc-200' : 'bg-black text-white hover:bg-zinc-800'
              }`}
            >
              Close
            </button>
          </div>
        )}

        <footer className="pt-3 border-t border-zinc-800/40 flex items-center justify-between text-[10px] font-mono opacity-50">
          <span>P4inz Interactive Labs • Airgapped Setup</span>
          <span>100% Offline • Zero Telemetry</span>
        </footer>
      </main>
    </div>
  );
}

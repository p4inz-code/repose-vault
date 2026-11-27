import React, { useState, useEffect } from 'react';
import {
  ArrowRight, AlertCircle, Fingerprint
} from 'lucide-react';
import type { PasskeyType, ThemeMode } from './lockTypes';
import type { OSPlatform } from './types';
import { ReposeLogo } from './ReposeLogo';
import { OSTitlebar } from './OSTitlebar';

interface LockScreenProps {
  os: OSPlatform;
  onSelectOS: (os: OSPlatform) => void;
  theme: ThemeMode;
  onToggleTheme: () => void;
  onAuthenticated: (key: string) => Promise<void> | void;
  authError?: string | null;
}

export function LockScreen({
  os,
  onSelectOS,
  theme,
  onToggleTheme,
  onAuthenticated,
  authError,
}: LockScreenProps) {
  // Modes: 'pin' (default 6-digit biometric style), 'password' (master passphrase), 'recovery'
  const [authMode, setAuthMode] = useState<PasskeyType>('pin');
  const [pinDigits, setPinDigits] = useState<string[]>([]);
  const [textPassword, setTextPassword] = useState<string>('');
  const [recoveryKey, setRecoveryKey] = useState<string>('');
  const [isVerifying, setIsVerifying] = useState<boolean>(false);
  const [isBiometricPrompt, setIsBiometricPrompt] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const isDark = theme === 'dark';

  const verifyAuth = async (credential: string) => {
    setIsVerifying(true);
    setErrorMessage(null);

    const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      try {
        await onAuthenticated(credential);
      } catch (err) {
        setErrorMessage(err instanceof Error ? err.message : String(err));
        setPinDigits([]);
      } finally {
        setIsVerifying(false);
      }
      return;
    }

    setTimeout(() => {
      setIsVerifying(false);
      if (
        credential === '123456' ||
        credential === 'DemoVaultPass123!' ||
        credential === 'demo-master-key-123' ||
        credential.length >= 6
      ) {
        onAuthenticated(credential);
      } else {
        setErrorMessage('Authentication rejected: signature mismatch.');
        setPinDigits([]);
      }
    }, 320);
  };

  const handlePinPress = (digit: string) => {
    if (pinDigits.length < 6) {
      const next = [...pinDigits, digit];
      setPinDigits(next);
      setErrorMessage(null);
      if (next.length === 6) {
        verifyAuth(next.join(''));
      }
    }
  };

  const handlePinBackspace = () => {
    setPinDigits(prev => prev.slice(0, -1));
  };

  useEffect(() => {
    if (authMode !== 'pin') return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (isVerifying) return;
      if (/^[0-9]$/.test(e.key)) {
        e.preventDefault();
        setPinDigits(prev => {
          if (prev.length >= 6) return prev;
          const next = [...prev, e.key];
          setErrorMessage(null);
          if (next.length === 6) {
            setTimeout(() => verifyAuth(next.join('')), 0);
          }
          return next;
        });
      } else if (e.key === 'Backspace') {
        e.preventDefault();
        setPinDigits(prev => prev.slice(0, -1));
      } else if (e.key === 'Escape') {
        e.preventDefault();
        setPinDigits([]);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  const handleBiometricUnlock = () => {
    const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      setErrorMessage('OS Biometric slot not enrolled yet. Unlock with your Master Passphrase or PIN.');
      return;
    }
    setIsBiometricPrompt(true);
    setTimeout(() => {
      setIsBiometricPrompt(false);
      onAuthenticated('biometric-hardware-attested-session');
    }, 900);
  };

  return (
    <div className={`h-screen w-screen relative overflow-hidden select-none flex flex-col justify-between transition-colors duration-200 ${
      isDark ? 'bg-[#090a0f] text-[#f2f2f7]' : 'bg-[#f8f9fa] text-[#1c1c1e]'
    }`}>
      {/* Top Native OS Titlebar Frame */}
      <OSTitlebar
        os={os}
        onSelectOS={onSelectOS}
        theme={theme}
        onToggleTheme={onToggleTheme}
        isLocked={true}
      />

      {/* Precision ambient subtle glow */}
      <div className={`absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[650px] h-[450px] rounded-full blur-[150px] pointer-events-none transition-opacity duration-500 ${
        isDark ? 'bg-zinc-700/10' : 'bg-zinc-300/30'
      }`} />

      {/* Main Lock Card */}
      <main className="flex-1 flex items-center justify-center p-6 z-20">
        <div className={`w-full max-w-[370px] rounded-2xl p-7 border transition-all ${
          isDark
            ? 'bg-zinc-900/85 border-zinc-800/90 shadow-[0_24px_50px_rgba(0,0,0,0.6)] backdrop-blur-xl'
            : 'bg-white/95 border-zinc-200/90 shadow-[0_20px_50px_rgba(0,0,0,0.06)] backdrop-blur-xl'
        }`}>
          <div className="flex flex-col items-center text-center space-y-4">
            
            {/* Logo and App Title */}
            <div className="flex flex-col items-center gap-2">
              <ReposeLogo size={36} />
              <div>
                <h1 className="text-sm font-semibold tracking-tight">Repose Space Vault</h1>
                <p className="text-[11px] opacity-50 mt-0.5">
                  {os === 'windows'
                    ? 'Windows 11 Native • WDA_MONITOR Protected'
                    : os === 'macos'
                    ? 'macOS Sonoma • Hardened Runtime'
                    : 'Linux Wayland • mlock() Memory Enclave'}
                </p>
              </div>
            </div>

            {/* Segmented Auth Mode Switcher */}
            <div className={`w-full flex items-center justify-center p-0.5 rounded-lg border text-xs ${
              isDark ? 'bg-zinc-950 border-zinc-800' : 'bg-zinc-100 border-zinc-200'
            }`}>
              <button
                onClick={() => { setAuthMode('pin'); setErrorMessage(null); }}
                className={`flex-1 py-1 rounded cursor-pointer font-medium transition-colors ${
                  authMode === 'pin'
                    ? (isDark ? 'bg-zinc-800 text-white' : 'bg-white text-black shadow-2xs')
                    : 'opacity-50 hover:opacity-100'
                }`}
              >
                Quick PIN
              </button>
              <button
                onClick={() => { setAuthMode('password'); setErrorMessage(null); }}
                className={`flex-1 py-1 rounded cursor-pointer font-medium transition-colors ${
                  authMode === 'password'
                    ? (isDark ? 'bg-zinc-800 text-white' : 'bg-white text-black shadow-2xs')
                    : 'opacity-50 hover:opacity-100'
                }`}
              >
                Passphrase
              </button>
              <button
                onClick={() => { setAuthMode('recovery'); setErrorMessage(null); }}
                className={`flex-1 py-1 rounded cursor-pointer font-medium transition-colors ${
                  authMode === 'recovery'
                    ? (isDark ? 'bg-zinc-800 text-white' : 'bg-white text-black shadow-2xs')
                    : 'opacity-50 hover:opacity-100'
                }`}
              >
                Recovery
              </button>
            </div>

            {/* Error Message */}
            {(errorMessage || authError) && (
              <div className="flex items-center gap-1.5 text-xs text-rose-500 font-medium">
                <AlertCircle className="w-3.5 h-3.5 shrink-0" />
                <span>{errorMessage || authError}</span>
              </div>
            )}

            {/* MODE 1: Quick PIN with Numeric Pad */}
            {authMode === 'pin' && (
              <div className="w-full space-y-4">
                {/* 6-Digit PIN Indicators */}
                <div className="flex justify-center items-center gap-3 py-1">
                  {[0, 1, 2, 3, 4, 5].map(idx => {
                    const isFilled = pinDigits.length > idx;
                    return (
                      <div
                        key={idx}
                        className={`w-3.5 h-3.5 rounded-full border transition-all duration-150 ${
                          isFilled
                            ? (isDark ? 'bg-white border-white scale-110' : 'bg-black border-black scale-110')
                            : (isDark ? 'border-zinc-700 bg-zinc-800/40' : 'border-zinc-300 bg-zinc-100')
                        }`}
                      />
                    );
                  })}
                </div>

                {/* 3x4 Numeric Keypad */}
                <div className="grid grid-cols-3 gap-2 w-full max-w-[240px] mx-auto">
                  {['1', '2', '3', '4', '5', '6', '7', '8', '9'].map(digit => (
                    <button
                      key={digit}
                      onClick={() => handlePinPress(digit)}
                      disabled={isVerifying}
                      className={`h-11 rounded-lg text-sm font-semibold font-mono border transition-all cursor-pointer active:scale-95 ${
                        isDark
                          ? 'bg-zinc-800/60 border-zinc-750 text-white hover:bg-zinc-700/80 hover:border-zinc-600'
                          : 'bg-zinc-100/80 border-zinc-200 text-zinc-900 hover:bg-zinc-200/90'
                      }`}
                    >
                      {digit}
                    </button>
                  ))}
                  {/* Biometric quick button */}
                  <button
                    onClick={handleBiometricUnlock}
                    disabled={isVerifying}
                    className={`h-11 rounded-lg flex items-center justify-center border transition-all cursor-pointer ${
                      isDark
                        ? 'bg-zinc-900/60 border-zinc-800 text-zinc-400 hover:text-white hover:bg-zinc-800'
                        : 'bg-zinc-50 border-zinc-200 text-zinc-600 hover:text-black hover:bg-zinc-100'
                    }`}
                    title={
                      os === 'windows'
                        ? 'Windows Hello Biometric'
                        : os === 'macos'
                        ? 'Touch ID'
                        : 'Linux Fingerprint'
                    }
                  >
                    <Fingerprint className="w-4 h-4 text-emerald-500" />
                  </button>
                  <button
                    onClick={() => handlePinPress('0')}
                    disabled={isVerifying}
                    className={`h-11 rounded-lg text-sm font-semibold font-mono border transition-all cursor-pointer active:scale-95 ${
                      isDark
                        ? 'bg-zinc-800/60 border-zinc-750 text-white hover:bg-zinc-700/80 hover:border-zinc-600'
                        : 'bg-zinc-100/80 border-zinc-200 text-zinc-900 hover:bg-zinc-200/90'
                    }`}
                  >
                    0
                  </button>
                  {/* Backspace */}
                  <button
                    onClick={handlePinBackspace}
                    disabled={isVerifying || pinDigits.length === 0}
                    className={`h-11 rounded-lg flex items-center justify-center text-xs font-mono border transition-all cursor-pointer disabled:opacity-30 ${
                      isDark
                        ? 'bg-zinc-900/60 border-zinc-800 text-zinc-400 hover:text-white hover:bg-zinc-800'
                        : 'bg-zinc-50 border-zinc-200 text-zinc-600 hover:text-black hover:bg-zinc-100'
                    }`}
                  >
                    ⌫
                  </button>
                </div>

                {/* OS Biometric Unlock Button */}
                <button
                  onClick={handleBiometricUnlock}
                  disabled={isVerifying || isBiometricPrompt}
                  className={`w-full py-2 px-3 rounded-lg border text-xs font-medium flex items-center justify-center gap-2 cursor-pointer transition-colors ${
                    isDark
                      ? 'bg-zinc-800/50 hover:bg-zinc-800 border-zinc-700/70 text-zinc-200'
                      : 'bg-zinc-100 hover:bg-zinc-200 border-zinc-300 text-zinc-800'
                  }`}
                >
                  <Fingerprint className="w-3.5 h-3.5 text-emerald-500" />
                  <span>
                    {isBiometricPrompt
                      ? 'Verifying Biometrics...'
                      : os === 'windows'
                      ? 'Unlock with Windows Hello'
                      : os === 'macos'
                      ? 'Unlock with Touch ID'
                      : 'Unlock with Linux fprintd'}
                  </span>
                </button>

                <p className="text-[10px] font-mono opacity-40">Demo PIN: 123456</p>
              </div>
            )}

            {/* MODE 2: Master Passphrase */}
            {authMode === 'password' && (
              <div className="w-full space-y-3">
                <div className="relative">
                  <input
                    type="password"
                    placeholder="Enter master passphrase..."
                    value={textPassword}
                    onChange={e => setTextPassword(e.target.value)}
                    onKeyDown={e => {
                      if (e.key === 'Enter' && textPassword) verifyAuth(textPassword);
                    }}
                    autoFocus
                    className={`w-full px-3.5 py-2.5 rounded-lg border text-xs font-mono transition-colors outline-hidden ${
                      isDark
                        ? 'bg-zinc-950/80 border-zinc-800 text-white placeholder-zinc-600 focus:border-zinc-600'
                        : 'bg-white border-zinc-300 text-black placeholder-zinc-400 focus:border-zinc-500'
                    }`}
                  />
                </div>

                <button
                  onClick={() => verifyAuth(textPassword)}
                  disabled={isVerifying || !textPassword}
                  className={`w-full py-2.5 rounded-lg font-medium text-xs flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-40 ${
                    isDark
                      ? 'bg-white hover:bg-zinc-200 text-black'
                      : 'bg-black hover:bg-zinc-800 text-white'
                  }`}
                >
                  {isVerifying ? (
                    <span>Deriving Argon2id keys...</span>
                  ) : (
                    <>
                      <span>Unlock Vault</span>
                      <ArrowRight className="w-3.5 h-3.5" />
                    </>
                  )}
                </button>
                <p className="text-[10px] font-mono opacity-40">Demo: DemoVaultPass123!</p>
              </div>
            )}

            {/* MODE 3: Recovery Key */}
            {authMode === 'recovery' && (
              <div className="w-full space-y-3">
                <textarea
                  rows={3}
                  placeholder="Enter 24-word recovery phrase or emergency key..."
                  value={recoveryKey}
                  onChange={e => setRecoveryKey(e.target.value)}
                  className={`w-full px-3 py-2 rounded-lg border text-xs font-mono resize-none transition-colors outline-hidden ${
                    isDark
                      ? 'bg-zinc-950/80 border-zinc-800 text-white placeholder-zinc-600 focus:border-zinc-600'
                      : 'bg-white border-zinc-300 text-black placeholder-zinc-400 focus:border-zinc-500'
                  }`}
                />

                <button
                  onClick={() => verifyAuth(recoveryKey)}
                  disabled={isVerifying || !recoveryKey}
                  className={`w-full py-2.5 rounded-lg font-medium text-xs flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-40 ${
                    isDark
                      ? 'bg-white hover:bg-zinc-200 text-black'
                      : 'bg-black hover:bg-zinc-800 text-white'
                  }`}
                >
                  {isVerifying ? (
                    <span>Reconstructing Master Key...</span>
                  ) : (
                    <>
                      <span>Recover Vault</span>
                      <ArrowRight className="w-3.5 h-3.5" />
                    </>
                  )}
                </button>
              </div>
            )}

          </div>
        </div>
      </main>

      {/* Footer Info */}
      <footer className="px-6 py-3 border-t flex items-center justify-between text-[11px] font-mono opacity-60 z-20 border-zinc-800/40">
        <div className="flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
          <span>Local Device Only • Zero Network Invariant</span>
        </div>
        <div className="flex items-center gap-3">
          <span>GPL v3.0</span>
          <span>Argon2id (64MB) + XChaCha20-Poly1305</span>
        </div>
      </footer>
    </div>
  );
}

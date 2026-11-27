// Simple client-side pseudo-TOTP generator for interactive UI preview simulation
export function generateMockTotp(secret?: string): { code: string; secondsLeft: number } {
  const epoch = Math.floor(Date.now() / 1000);
  const timeStep = 30;
  const secondsLeft = timeStep - (epoch % timeStep);
  
  if (!secret) return { code: '------', secondsLeft };
  
  // Deterministic 6 digit hash-like generator based on epoch window
  const windowIndex = Math.floor(epoch / timeStep);
  let hash = 0;
  for (let i = 0; i < secret.length; i++) {
    hash = (hash << 5) - hash + secret.charCodeAt(i);
    hash |= 0;
  }
  const codeNum = Math.abs((hash ^ windowIndex * 31337) % 900000) + 100000;
  const str = codeNum.toString();
  return { code: `${str.slice(0, 3)} ${str.slice(3)}`, secondsLeft };
}

export function generatePassword(options: {
  length: number;
  useUpper: boolean;
  useLower: boolean;
  useNumbers: boolean;
  useSymbols: boolean;
  avoidAmbiguous: boolean;
}): string {
  let upper = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
  let lower = 'abcdefghijklmnopqrstuvwxyz';
  let numbers = '0123456789';
  let symbols = '!@#$%^&*()_+-=[]{}|;:,.<>?';

  if (options.avoidAmbiguous) {
    upper = upper.replace(/[IO]/g, '');
    lower = lower.replace(/[lo]/g, '');
    numbers = numbers.replace(/[01]/g, '');
    symbols = symbols.replace(/[|;:,.<>]/g, '');
  }

  let charset = '';
  if (options.useUpper) charset += upper;
  if (options.useLower) charset += lower;
  if (options.useNumbers) charset += numbers;
  if (options.useSymbols) charset += symbols;

  if (!charset) charset = lower + numbers;

  let result = '';
  const array = new Uint32Array(options.length);
  window.crypto.getRandomValues(array);
  for (let i = 0; i < options.length; i++) {
    result += charset[array[i] % charset.length];
  }
  return result;
}

export function calculatePasswordStrength(pass: string): {
  score: number; // 0 to 4
  label: 'Weak' | 'Fair' | 'Good' | 'Strong' | 'Unbreakable';
  color: string;
} {
  if (!pass) return { score: 0, label: 'Weak', color: '#ef4444' };
  let points = 0;
  if (pass.length >= 8) points++;
  if (pass.length >= 14) points++;
  if (pass.length >= 20) points++;
  if (/[A-Z]/.test(pass) && /[a-z]/.test(pass)) points++;
  if (/[0-9]/.test(pass) && /[^A-Za-z0-9]/.test(pass)) points++;

  if (points <= 1) return { score: 1, label: 'Weak', color: '#ef4444' };
  if (points === 2) return { score: 2, label: 'Fair', color: '#f59e0b' };
  if (points === 3) return { score: 3, label: 'Good', color: '#eab308' };
  if (points === 4) return { score: 4, label: 'Strong', color: '#10b981' };
  return { score: 4, label: 'Unbreakable', color: '#06b6d4' };
}

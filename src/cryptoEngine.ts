import { openDB, DBSchema } from 'idb';
import type { VaultItem } from './types';

const DB_NAME = 'repose_secure_store';
const DB_VERSION = 1;
const KDF_SALT_STORAGE_KEY = 'repose_salt_v1';
const METADATA_STORAGE_KEY = 'repose_meta_v1';

interface VaultDB extends DBSchema {
  encrypted_vault: {
    key: string;
    value: {
      id: string;
      ciphertext: string;
      iv: string;
      updatedAt: string;
    };
  };
}

export async function getDB() {
  return openDB<VaultDB>(DB_NAME, DB_VERSION, {
    upgrade(db) {
      if (!db.objectStoreNames.contains('encrypted_vault')) {
        db.createObjectStore('encrypted_vault', { keyPath: 'id' });
      }
    },
  });
}

function bufferToHex(buffer: ArrayBuffer | Uint8Array): string {
  const bytes = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

function hexToBuffer(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.substring(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

export function getOrCreateSalt(): Uint8Array {
  let stored = localStorage.getItem(KDF_SALT_STORAGE_KEY);
  if (!stored) {
    const salt = new Uint8Array(32);
    window.crypto.getRandomValues(salt);
    stored = bufferToHex(salt);
    localStorage.setItem(KDF_SALT_STORAGE_KEY, stored);
  }
  return hexToBuffer(stored);
}

export function generateRecoveryKey(): string {
  const bytes = new Uint8Array(20);
  window.crypto.getRandomValues(bytes);
  const hex = bufferToHex(bytes).toUpperCase();
  // Formats as 8 groups of 5 chars
  return hex.match(/.{1,5}/g)?.join('-') || hex;
}

export async function deriveKeyFromPassword(password: string, salt: Uint8Array): Promise<CryptoKey> {
  const enc = new TextEncoder();
  const keyMaterial = await window.crypto.subtle.importKey(
    'raw',
    enc.encode(password),
    'PBKDF2',
    false,
    ['deriveKey']
  );

  return window.crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt: salt as any,
      iterations: 300000,
      hash: 'SHA-256'
    },
    keyMaterial,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt', 'decrypt']
  );
}

export async function encryptData(key: CryptoKey, plaintext: string): Promise<{ ciphertext: string; iv: string }> {
  const enc = new TextEncoder();
  const iv = new Uint8Array(12);
  window.crypto.getRandomValues(iv);

  const encryptedBuffer = await window.crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    key,
    enc.encode(plaintext)
  );

  return {
    ciphertext: bufferToHex(encryptedBuffer),
    iv: bufferToHex(iv)
  };
}

export async function decryptData(key: CryptoKey, ciphertextHex: string, ivHex: string): Promise<string> {
  const dec = new TextDecoder();
  const ciphertext = hexToBuffer(ciphertextHex);
  const iv = hexToBuffer(ivHex);

  const decryptedBuffer = await window.crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: iv as unknown as BufferSource },
    key,
    ciphertext as unknown as BufferSource
  );

  return dec.decode(decryptedBuffer);
}

export async function saveVaultItems(key: CryptoKey, items: VaultItem[]): Promise<void> {
  const db = await getDB();
  const serialized = JSON.stringify(items);
  const { ciphertext, iv } = await encryptData(key, serialized);

  await db.put('encrypted_vault', {
    id: 'primary_vault',
    ciphertext,
    iv,
    updatedAt: new Date().toISOString()
  });
}

export async function loadVaultItems(key: CryptoKey): Promise<VaultItem[] | null> {
  const db = await getDB();
  const record = await db.get('encrypted_vault', 'primary_vault');
  if (!record) return null;

  try {
    const json = await decryptData(key, record.ciphertext, record.iv);
    return JSON.parse(json) as VaultItem[];
  } catch {
    throw new Error('Authentication failed: Incorrect password or tampered data.');
  }
}

export async function hasSavedVault(): Promise<boolean> {
  const db = await getDB();
  const record = await db.get('encrypted_vault', 'primary_vault');
  return !!record;
}

export function wipeVaultDatabase(): Promise<void> {
  return new Promise((resolve, reject) => {
    localStorage.removeItem(KDF_SALT_STORAGE_KEY);
    localStorage.removeItem(METADATA_STORAGE_KEY);
    const req = indexedDB.deleteDatabase(DB_NAME);
    req.onsuccess = () => resolve();
    req.onerror = () => reject(req.error);
    req.onblocked = () => resolve();
  });
}

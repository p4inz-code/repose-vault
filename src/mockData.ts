import type { Collection, VaultItem } from './types';

export const COLLECTIONS: Collection[] = [
  { id: 'all', name: 'All Secrets' },
  { id: 'infrastructure', name: 'Infrastructure & Servers' },
  { id: 'access', name: 'Credentials & Access' },
  { id: 'finance', name: 'Financial Records' },
  { id: 'identities', name: 'Secure Identities' },
  { id: 'offline', name: 'Airgapped Cold Data' },
];

export const INITIAL_ITEMS: VaultItem[] = [
  {
    id: 'vault-item-1',
    category: 'server',
    title: 'Primary Linux Gateway (SSH Ed25519)',
    collectionId: 'infrastructure',
    favorite: true,
    primaryIdentifier: 'root@10.0.4.1',
    primarySecret: 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGt1M... root@gateway-internal',
    fields: [
      { id: 'f-1', label: 'Host Address', value: '10.0.4.1:2222', isSecret: false, fieldKind: 'text' },
      { id: 'f-2', label: 'SSH User', value: 'bastion-admin', isSecret: false, fieldKind: 'text' },
      { id: 'f-3', label: 'Passphrase', value: 'p@ss-B@st1on-K3y-2026', isSecret: true, fieldKind: 'secret' },
      { id: 'f-4', label: 'Fingerprint', value: 'SHA256:d8q129f9mZkQp91002A11b', isSecret: false, fieldKind: 'text' },
    ],
    notes: 'Access restricted to internal subnet. Jump host routing: bastion -> core.',
    createdAt: '2026-10-15T09:20:00Z',
    updatedAt: '2026-11-20T14:15:00Z'
  },
  {
    id: 'vault-item-2',
    category: 'database',
    title: 'Production PostgreSQL Cluster',
    collectionId: 'infrastructure',
    favorite: false,
    primaryIdentifier: 'postgres_master',
    primarySecret: 'pg_sec_99a80b17c_xL901p!',
    fields: [
      { id: 'f-db-1', label: 'Host', value: 'db-cluster-primary.internal', isSecret: false, fieldKind: 'text' },
      { id: 'f-db-2', label: 'Port', value: '5432', isSecret: false, fieldKind: 'text' },
      { id: 'f-db-3', label: 'Database', value: 'core_ledger_production', isSecret: false, fieldKind: 'text' },
      { id: 'f-db-4', label: 'Connection URI', value: 'postgresql://postgres_master:***@db-cluster-primary.internal:5432/core_ledger_production', isSecret: true, fieldKind: 'secret' }
    ],
    notes: 'Replica nodes: 10.0.8.20, 10.0.8.21. SSL Mode: verify-full.',
    createdAt: '2026-09-01T10:00:00Z',
    updatedAt: '2026-11-25T11:40:00Z'
  },
  {
    id: 'vault-item-3',
    category: 'login',
    title: 'Administrative Web Console',
    collectionId: 'access',
    favorite: true,
    primaryIdentifier: 'admin@system.local',
    primarySecret: 'kR#9$mX!92vLz@8qP~11',
    fields: [
      { id: 'f-l-1', label: 'URL', value: 'https://console.internal.corp', isSecret: false, fieldKind: 'url' },
      { id: 'f-l-2', label: 'TOTP Secret', value: 'HXDMVJECJJWSRB3HWIZR4IFUGFTMXBOZ', isSecret: true, fieldKind: 'totp' },
      { id: 'f-l-3', label: 'Backup Codes', value: '8491-0294, 9182-3819, 0192-3810', isSecret: true, fieldKind: 'secret' }
    ],
    notes: 'Mandatory hardware verification every 14 days.',
    createdAt: '2026-11-01T08:10:00Z',
    updatedAt: '2026-11-01T08:10:00Z'
  },
  {
    id: 'vault-item-4',
    category: 'token',
    title: 'Hardware Cryptographic Signing Key',
    collectionId: 'offline',
    favorite: false,
    primaryIdentifier: 'ID-HSM-88910',
    primarySecret: 'secp256k1:04a91f88c9102831e90184b910281b901a8849',
    fields: [
      { id: 'f-h-1', label: 'Key Slot', value: 'Slot 01 - Firmware Root', isSecret: false, fieldKind: 'text' },
      { id: 'f-h-2', label: 'PIN', value: '98401928', isSecret: true, fieldKind: 'secret' },
      { id: 'f-h-3', label: 'PUK Code', value: '001928491029', isSecret: true, fieldKind: 'secret' }
    ],
    notes: 'Cold storage unit located in Vault Safe B.',
    createdAt: '2026-07-20T18:30:00Z',
    updatedAt: '2026-11-26T21:05:00Z'
  },
  {
    id: 'vault-item-5',
    category: 'card',
    title: 'Operating Reserve Account Card',
    collectionId: 'finance',
    favorite: true,
    primaryIdentifier: 'ATHARVA PATIL',
    primarySecret: '4532 8901 2940 9821',
    fields: [
      { id: 'f-c-1', label: 'Expiry Date', value: '08/30', isSecret: false, fieldKind: 'text' },
      { id: 'f-c-2', label: 'Security Code (CVV)', value: '849', isSecret: true, fieldKind: 'secret' },
      { id: 'f-c-3', label: 'ATM PIN', value: '7192', isSecret: true, fieldKind: 'secret' },
      { id: 'f-c-4', label: 'Billing Address', value: 'Suite 400, Financial District', isSecret: false, fieldKind: 'text' }
    ],
    notes: 'Corporate expense authorization required above limit.',
    createdAt: '2026-08-14T11:00:00Z',
    updatedAt: '2026-10-02T16:22:00Z'
  },
  {
    id: 'vault-item-6',
    category: 'recovery',
    title: 'Disaster Recovery Airgap Seed',
    collectionId: 'offline',
    favorite: true,
    primaryIdentifier: 'COLD-SEED-24W',
    primarySecret: 'timber ocean marble crystal glacier velvet pulse quantum beacon orbit granite falcon summit shadow canyon cobalt ember vortex prism zenith aurora tempest anchor nexus',
    fields: [
      { id: 'f-r-1', label: 'Format', value: 'BIP-39 / 256-bit Entropy', isSecret: false, fieldKind: 'text' },
      { id: 'f-r-2', label: 'Derivation Path', value: "m/44'/0'/0'/0", isSecret: false, fieldKind: 'text' }
    ],
    notes: 'Never digitize on internet-connected hosts.',
    createdAt: '2026-06-11T14:10:00Z',
    updatedAt: '2026-11-12T19:30:00Z'
  }
];

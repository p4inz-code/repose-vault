export type ItemCategory = 
  | 'login'
  | 'token'
  | 'database'
  | 'server'
  | 'card'
  | 'identity'
  | 'document'
  | 'recovery';

export interface VaultField {
  id: string;
  label: string;
  value: string;
  isSecret: boolean;
  fieldKind?: 'text' | 'secret' | 'multiline' | 'totp' | 'date' | 'url';
}

export interface VaultItem {
  id: string;
  category: ItemCategory;
  title: string;
  collectionId: string;
  favorite: boolean;
  isTrash?: boolean;
  deletedAt?: string;
  
  // Primary identifier (e.g., username, host, email, cardholder)
  primaryIdentifier?: string;
  // Primary secret (e.g., password, private key, token, PIN)
  primarySecret?: string;
  
  // Structured custom & standard fields for any format
  fields: VaultField[];
  
  // Secure encrypted markdown notes
  notes?: string;
  
  createdAt: string;
  updatedAt: string;
}

export interface Collection {
  id: string;
  name: string;
}

export type OSPlatform = 'windows' | 'macos' | 'linux';
export type ThemeMode = 'dark' | 'light';

export type PasskeyType = 'pin' | 'pattern' | 'password' | 'recovery';

export interface ChallengeGate {
  id: string;
  type: 'spatial_slider' | 'geometric_pattern' | 'chronos_dial' | 'glyph_sequence';
  title: string;
  instruction: string;
}

export type ThemeMode = 'dark' | 'light';

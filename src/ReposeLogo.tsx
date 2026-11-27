import React from 'react';

interface ReposeLogoProps {
  className?: string;
  size?: number;
}

export function ReposeLogo({ className = '', size = 26 }: ReposeLogoProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={`shrink-0 ${className}`}
    >
      {/* Container squircle */}
      <rect
        x="1.5"
        y="1.5"
        width="29"
        height="29"
        rx="7"
        className="fill-zinc-900 dark:fill-zinc-950 stroke-zinc-700 dark:stroke-zinc-700"
        strokeWidth="1.25"
      />
      
      {/* Precision kinetic vault arch */}
      <path
        d="M9.5 16.5C9.5 12.91 12.41 10 16 10C19.59 10 22.5 12.91 22.5 16.5"
        stroke="#ffffff"
        strokeWidth="1.75"
        strokeLinecap="round"
      />
      
      {/* Core airgapped key node */}
      <circle
        cx="16"
        cy="16.5"
        r="2.5"
        fill="#ffffff"
      />
      
      {/* Stable ground baseline */}
      <path
        d="M10.5 22.5H21.5"
        stroke="#888888"
        strokeWidth="1.75"
        strokeLinecap="round"
      />
    </svg>
  );
}

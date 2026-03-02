'use client';

import { FC } from 'react';

interface TrustBadgeProps {
  score: number; // 0-10000
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
}

export const TrustBadge: FC<TrustBadgeProps> = ({ score, size = 'md', showLabel = true }) => {
  const displayScore = (score / 100).toFixed(1);
  const stars = Math.round(score / 2000); // 0-5 stars

  const colorClass =
    score >= 8000
      ? 'text-forkit-green bg-forkit-green/10 border-forkit-green/30'
      : score >= 5000
        ? 'text-forkit-amber bg-forkit-amber/10 border-forkit-amber/30'
        : 'text-forkit-red bg-forkit-red/10 border-forkit-red/30';

  const sizeClass = {
    sm: 'text-xs px-2 py-0.5',
    md: 'text-sm px-3 py-1',
    lg: 'text-base px-4 py-1.5',
  }[size];

  return (
    <span className={`inline-flex items-center gap-1 rounded-full border font-medium ${colorClass} ${sizeClass}`}>
      {'★'.repeat(stars)}{'☆'.repeat(5 - stars)}
      {showLabel && <span className="ml-1">{displayScore}</span>}
    </span>
  );
};

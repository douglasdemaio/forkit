'use client';

import { FC, useState, useRef } from 'react';

interface CodeInputProps {
  length?: number;
  onComplete: (code: string) => void;
  label: string;
  disabled?: boolean;
}

export const CodeInput: FC<CodeInputProps> = ({ length = 6, onComplete, label, disabled = false }) => {
  const [values, setValues] = useState<string[]>(Array(length).fill(''));
  const inputRefs = useRef<(HTMLInputElement | null)[]>([]);

  const handleChange = (index: number, value: string) => {
    const char = value.toUpperCase().slice(-1);
    if (char && !/[A-Z0-9]/.test(char)) return;

    const newValues = [...values];
    newValues[index] = char;
    setValues(newValues);

    if (char && index < length - 1) {
      inputRefs.current[index + 1]?.focus();
    }

    const code = newValues.join('');
    if (code.length === length && !code.includes('')) {
      onComplete(code);
    }
  };

  const handleKeyDown = (index: number, e: React.KeyboardEvent) => {
    if (e.key === 'Backspace' && !values[index] && index > 0) {
      inputRefs.current[index - 1]?.focus();
    }
  };

  return (
    <div className="space-y-2">
      <label className="block text-sm font-medium text-gray-300">{label}</label>
      <div className="flex gap-2 justify-center">
        {values.map((val, i) => (
          <input
            key={i}
            ref={(el) => { inputRefs.current[i] = el; }}
            type="text"
            maxLength={1}
            value={val}
            disabled={disabled}
            onChange={(e) => handleChange(i, e.target.value)}
            onKeyDown={(e) => handleKeyDown(i, e)}
            className="w-12 h-14 text-center text-2xl font-mono font-bold
              bg-forkit-slate border-2 border-gray-600 rounded-lg
              text-white focus:border-forkit-green focus:outline-none
              disabled:opacity-50 disabled:cursor-not-allowed
              transition-colors"
          />
        ))}
      </div>
    </div>
  );
};

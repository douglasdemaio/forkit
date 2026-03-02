import type { Config } from 'tailwindcss';

const config: Config = {
  content: [
    './app/**/*.{js,ts,jsx,tsx,mdx}',
    './components/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        forkit: {
          green: '#10B981',
          'green-dark': '#059669',
          amber: '#F59E0B',
          red: '#EF4444',
          navy: '#0F172A',
          slate: '#1E293B',
        },
      },
    },
  },
  plugins: [],
};

export default config;

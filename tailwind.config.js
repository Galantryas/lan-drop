/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{html,js,ts,svelte}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        dark: {
          900: '#0b0f19',
          800: '#111827',
          700: '#1f2937',
          600: '#374151'
        },
        brand: {
          500: '#3b82f6',
          600: '#2563eb',
          400: '#60a5fa'
        }
      }
    },
  },
  plugins: [],
};

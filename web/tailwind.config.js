/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        green: {
          500: '#10b981',
        },
        yellow: {
          500: '#eab308',
        },
        red: {
          500: '#ef4444',
        },
        gray: {
          500: '#6b7280',
        },
      },
    },
  },
  plugins: [],
}

/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: '#08006C',
          light: '#13098C',
          dark: '#05004A',
        },
        accent: {
          DEFAULT: '#FD5901',
          hover: '#E04E00',
          light: '#FFF0E8',
        },
        darkbg: {
          base: '#12131A',
          surface: '#1D1F2B',
          secondary: '#262938',
          border: '#32364A',
        },
        darktext: {
          base: '#F5F6FA',
          muted: '#A6AABD',
        }
      }
    },
  },
  plugins: [],
}

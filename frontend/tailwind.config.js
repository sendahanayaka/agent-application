/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['"Google Sans"', '"Segoe UI"', 'sans-serif'],
      },
      colors: {
        primary: {
          DEFAULT: '#14634e',
          light: '#0e5240',
          dark: '#0c4036',
        },
        accent: {
          DEFAULT: '#FD5901',
          hover: '#E04E00',
          light: '#FFF0E8',
        },
        darkbg: {
          base: '#142019',
          surface: '#19251f',
          secondary: '#223028',
          border: '#35473b',
        },
        darktext: {
          base: '#e4eee6',
          muted: '#b2c2b8',
        }
      }
    },
  },
  plugins: [],
}

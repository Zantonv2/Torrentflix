/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    colors: {
      transparent: 'transparent',
      current: 'currentColor',
      white: '#FFFFFF',
      black: '#000000',
      blue: {
        600: '#2563eb',
      },
      green: {
        600: '#16a34a',
      },
      yellow: {
        600: '#ca8a04',
      },
      gray: {
        600: '#4b5563',
      },
      netflix: {
        black: '#141414',
        dark: '#181818',
        gray: '#333333',
        light: '#757575',
        red: '#E50914',
        white: '#FFFFFF',
      },
    },
    extend: {
      fontFamily: {
        netflix: ['"Netflix Sans"', 'Helvetica Neue', 'sans-serif'],
      },
      backdropBlur: {
        sm: '4px',
        md: '8px',
        lg: '12px',
      },
      backgroundColor: {
        'theme-primary': 'var(--bg-primary, #0f0f0f)',
        'theme-secondary': 'var(--bg-secondary, #141414)',
        'theme-tertiary': 'var(--bg-tertiary, #1a1a1a)',
        'accent': 'var(--accent-color, #E50914)',
      },
      textColor: {
        'theme-primary': 'var(--text-primary, #ffffff)',
        'theme-secondary': 'var(--text-secondary, #b3b3b3)',
        'accent': 'var(--accent-color, #E50914)',
      },
      borderColor: {
        'accent': 'var(--accent-color, #E50914)',
      },
      animation: {
        'fade-in': 'fadeIn 0.3s ease-in-out',
        'slide-up': 'slideUp 0.3s ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { transform: 'translateY(20px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
      },
    },
  },
  plugins: [],
  future: {
    hoverOnlyWhenSupported: true,
  },
}

import type { Preview } from '@storybook/react-vite'
import '../src/styles/globals.css'

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    backgrounds: {
      options: {
        light: { name: 'light', value: '#FFFFFF' },
        dark: { name: 'dark', value: '#171717' },
        gray: { name: 'gray', value: '#F3F3F3' },
      },
    },
  },

  initialGlobals: {
    backgrounds: {
      value: 'light',
    },
  },
}

export default preview

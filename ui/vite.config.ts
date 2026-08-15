import { defineConfig } from 'vite';
import { devtools } from '@tanstack/devtools-vite';

import { tanstackRouter } from '@tanstack/router-plugin/vite';

import viteReact from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

const serverURL = 'http://localhost:' + (process.env.PORT ?? '3000');

const config = defineConfig({
  resolve: { tsconfigPaths: true },
  base: process.env.UI_BASE_URL ?? '/',
  plugins: [
    devtools(),
    tailwindcss(),
    tanstackRouter({ target: 'react', autoCodeSplitting: true }),
    viteReact(),
  ],
  server: {
    proxy: {
      // this routes /api requests to the locally running api backend in dev
      '/api': serverURL,
      '/apidoc': serverURL,
      '/auth': serverURL,
      '/swagger-ui': serverURL,
    },
  },
});

export default config;

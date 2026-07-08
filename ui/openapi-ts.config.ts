import { defineConfig } from '@hey-api/openapi-ts';

export default defineConfig({
  input: '../openapi/schema.json', // sign up at app.heyapi.dev
  output: 'src/client',
  plugins: ['@tanstack/react-query'],
});

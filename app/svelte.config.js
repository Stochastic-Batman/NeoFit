import adapter from '@sveltejs/adapter-auto';
import { resolve } from 'path';
import { fileURLToPath } from 'url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
		runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true)
	},
	kit: {
		adapter: adapter(),
		alias: {
			// Points one level up to the workspace-root exercises.json.
			// SvelteKit propagates this alias to both Vite and the
			// auto-generated .svelte-kit/tsconfig.json, so no separate
			// edits to vite.config.ts or tsconfig.json are needed.
			$exercises: resolve(__dirname, '../exercises.json')
		}
	}
};

export default config;

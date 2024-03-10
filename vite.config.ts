import { sveltekit } from '@sveltejs/kit/vite';
import wasm from 'vite-plugin-wasm';
import wasmPack from 'vite-plugin-wasm-pack';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit(), wasmPack('./bampy'), wasm()],
	ssr: {
		noExternal: ['three']
	}
});

<script lang="ts">
	import { Canvas } from '@threlte/core';
	import Scene from './Scene.svelte';

	let progress = 1;
</script>

<Canvas>
	<Scene bind:progress />
</Canvas>

<div class="controls">
	<input type="range" min="0" max="1" step="0.01" bind:value={progress} />
</div>

<style>
	.controls {
		position: absolute;
		top: 0;
		right: 0;
	}
</style>

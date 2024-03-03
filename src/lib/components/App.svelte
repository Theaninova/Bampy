<script lang="ts">
	import { Canvas } from '@threlte/core';
	import Scene from './Scene.svelte';

	let progress = 1;
	let showSlices = 1;
	let progressLayer = 0;
</script>

<Canvas>
	<Scene bind:progress bind:showSlices bind:progressLayer />
</Canvas>

<div class="controls">
	<input type="range" min="0" max="1" step="0.01" bind:value={showSlices} />
	<label>Layer {progressLayer} <progress value={progress} /></label>
</div>

<style>
	.controls {
		display: flex;
		flex-direction: column;
		position: absolute;
		top: 0;
		right: 0;
	}
	label {
		color: white;
	}
</style>

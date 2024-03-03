<script lang="ts">
	import { Canvas } from '@threlte/core';
	import Scene from './Scene.svelte';
	import type { Writable } from 'svelte/store';
	import { fly } from 'svelte/transition';

	let progress: Writable<number | undefined>;
	let showSlices = 1;
	let progressLayer: Writable<number>;
</script>

<Canvas>
	<Scene bind:progress bind:showSlices bind:progressLayer />
</Canvas>

<div class="controls">
	<input type="range" min="0" max="1" step="0.01" bind:value={showSlices} orient="vertical" />
</div>

{#if $progress !== undefined}
	<div class="progress" transition:fly={{ y: 10 }}>
		<div class="progress-card">
			Layer {$progressLayer ?? 0}
			<progress value={$progress ?? 0} />
		</div>
	</div>
{/if}

<style lang="scss">
	input[type='range'] {
	}

	.controls {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		position: absolute;
		top: 0;
		bottom: 0;
		right: 0;
		margin: 10px;
	}

	.progress {
		display: flex;
		flex-direction: column;
		position: absolute;
		bottom: 0;
		justify-content: center;
		align-items: center;
	}

	progress {
		width: 100%;
		background: transparent;
		border: 1px solid white;
		color: white;
		border-radius: 4px;

		&::-webkit-progress-bar {
			background: transparent;
		}
	}

	.progress-card {
		margin: 14px;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		gap: 2px;
		background: #22aaee;
		color: white;
		padding: 10px;
		border-radius: 14px;
	}
</style>

<script lang="ts">
	import { T, type AsyncWritable } from '@threlte/core';
	import { Gizmo, Grid, OrbitControls, MeshLineGeometry, MeshLineMaterial } from '@threlte/extras';
	import { STLLoader } from 'three/examples/jsm/loaders/STLLoader.js';
	import { useLoader } from '@threlte/core';
	import {
		BufferGeometry,
		MathUtils,
		Vector3,
		DoubleSide,
		Color,
		BufferGeometryLoader
	} from 'three';
	import { writable } from 'svelte/store';
	import { onDestroy, onMount } from 'svelte';
	import SliceWorker from '$lib/slicer/worker.js?worker';
	import {
		LayerType,
		type WorkerMessage,
		type Layer,
		type SliceEvent
	} from '$lib/slicer/worker-data';

	let sliceWorker: Worker;

	onMount(() => {
		const geometryLoader = new BufferGeometryLoader();
		sliceWorker = new SliceWorker();
		sliceWorker.addEventListener('message', (event: MessageEvent<WorkerMessage>) => {
			switch (event.data.type) {
				case 'progress': {
					progress.set(event.data.percent);
					progressLayer.set(event.data.layer);
					break;
				}
				case 'result': {
					layers.update((layers) => {
						for (const layer of event.data.data.slices) {
							if (layer.type === 'ring' || layer.type === 'path') {
								layers.push(
									Array.from({ length: layer.position.length / 3 }, (_, i) =>
										new Vector3().fromArray(layer.position, i * 3)
									)
								);
							} else if (layer.type === 'surface') {
							}
						}
						return layers;
					});
					break;
				}
			}
		});
	});

	onDestroy(() => {
		sliceWorker?.terminate();
	});

	export let buildSurface = [300, 300, 300];
	export let layerHeight = 0.2;
	export let nozzleDiameter = 0.4;
	export let tolerance = 0.005;
	export let progress = writable<number | undefined>(undefined);
	export let progressLayer = writable(0);
	export let showSlices = 1;
	export let maxZ = 0;

	export let maxNonPlanarAngle = MathUtils.degToRad(20);
	export let bedNormal = new Vector3(0, 0, 1);

	let layers = writable<Vector3[][]>([]);

	const stl: AsyncWritable<BufferGeometry> = useLoader(STLLoader).load('/benchy.stl');

	$: if ($stl) {
		sliceWorker.postMessage({
			type: 'slice',
			data: {
				stl: $stl.toJSON(),
				layerHeight,
				tolerance,
				maxNonPlanarAngle,
				nozzleDiameter,
				minSurfacePathLength: nozzleDiameter * 2,
				bedNormal: bedNormal.toArray()
			}
		} satisfies SliceEvent);
	}
</script>

<T.PerspectiveCamera makeDefault position={buildSurface} fov={60} up={[0, 0, 1]}>
	<OrbitControls enableDamping target.y={1.5} />
</T.PerspectiveCamera>

<Gizmo />

<Grid
	position.y={-0.001}
	cellColor="#ffffff"
	sectionColor="#ff1111"
	fadeDistance={buildSurface[0] * 2}
	cellSize={10}
	sectionSize={0}
	plane="xy"
	gridSize={[buildSurface[0], buildSurface[1]]}
/>

{#each $layers as points, i}
	{@const visible = maxZ !== 0 ? i === maxZ : showSlices >= i / $layers.length}
	{@const color = new Color(Math.random() * 0xffffff)}
	<!---{@const color = new Color(0, i / $layers.length, 0.2)}-->
	<T.Mesh {visible}>
		<MeshLineGeometry {points} />
		<MeshLineMaterial width={nozzleDiameter * 0.25} {color} />
	</T.Mesh>
{/each}

{#if $stl}
	<T.Mesh geometry={$stl} position.y={40}>
		<T.MeshNormalMaterial />
	</T.Mesh>
{/if}

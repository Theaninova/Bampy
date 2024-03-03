<script lang="ts">
	import { T } from '@threlte/core';
	import { Gizmo, Grid, OrbitControls } from '@threlte/extras';
	import { STLLoader } from 'three/examples/jsm/loaders/STLLoader';
	import { useLoader } from '@threlte/core';
	import {
		BufferGeometry,
		MathUtils,
		Vector3,
		Mesh,
		DoubleSide,
		Color,
		Box3,
		Matrix4
	} from 'three';
	import { ExtendedTriangle, MeshBVH } from 'three-mesh-bvh';
	import type { Readable } from 'svelte/store';
	import type { HitPointInfo } from 'three-mesh-bvh';
	import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';

	export let buildSurface = [300, 300, 300];
	export let layerHeight = 0.2;
	export let nozzleSize = 0.4;
	export let tolerance = 0.005;

	export let maxNonPlanarAngle = MathUtils.degToRad(20);
	export let bedNormal = new Vector3(0, 0, 1);
	export let origin = new Vector3(150, 150, 0);

	const stl: Readable<BufferGeometry> = useLoader(STLLoader).load('/benchy.stl');

	let mesh: Mesh;
	let layers: BufferGeometry[] = [];

	$: if ($stl) {
		const bvh = new MeshBVH($stl);
		const positions = $stl.getAttribute('position');
		const index = $stl.index!;

		const qualifyingTriangles = Array.from({ length: index.count / 3 }, () => false);
		let qualifyingTrianglesCount = 0;
		const triangle = new ExtendedTriangle();
		const normal = new Vector3();
		for (let i = 0; i < index.count / 3; i++) {
			triangle.setFromAttributeAndIndices(
				positions,
				index.array[i * 3],
				index.array[i * 3 + 1],
				index.array[i * 3 + 2]
			);
			triangle.getNormal(normal);
			const angle = normal.angleTo(bedNormal);
			if ((angle > Math.PI / 2 ? Math.PI - angle : angle) < maxNonPlanarAngle) {
				qualifyingTriangles[i] = true;
				qualifyingTrianglesCount++;
			}
		}

		const surfaces: number[][] = [];
		while (qualifyingTrianglesCount > 0) {
			const faceIndex = qualifyingTriangles.findIndex((it) => it);
			qualifyingTriangles[faceIndex] = false;
			qualifyingTrianglesCount--;
			const surface = [faceIndex];
			let cursor = 0;
			while (cursor < surface.length) {
				triangle.setFromAttributeAndIndices(
					positions,
					index.array[surface[cursor] * 3],
					index.array[surface[cursor] * 3 + 1],
					index.array[surface[cursor] * 3 + 2]
				);

				bvh.shapecast({
					intersectsBounds(box, _isLeaf, _score, _depth, _nodeIndex) {
						return triangle.intersectsBox(box);
					},
					intersectsTriangle(target, triangleIndex, _contained, _depth) {
						if (
							qualifyingTriangles[triangleIndex] &&
							target.distanceToTriangle(triangle) < tolerance
						) {
							qualifyingTriangles[triangleIndex] = false;
							qualifyingTrianglesCount--;
							surface.push(triangleIndex);
						}
					}
				});

				cursor++;
			}
			surfaces.push(surface);
		}

		layers = surfaces.map((surface) => {
			const geometry = new BufferGeometry();
			geometry.setAttribute('position', positions);
			geometry.setAttribute('normal', $stl.getAttribute('normal'));
			const indices: number[] = Array.from({ length: surface.length * 3 });
			for (let i = 0; i < surface.length; i++) {
				const pos = surface[i] * 3;
				indices[i * 3] = $stl.index!.array[pos];
				indices[i * 3 + 1] = $stl.index!.array[pos + 1];
				indices[i * 3 + 2] = $stl.index!.array[pos + 2];
			}
			geometry.setIndex(indices);
			return geometry;
		});
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

{#each layers as surface}
	<T.Mesh geometry={surface}>
		<T.MeshMatcapMaterial color={new Color(Math.random() * 0xffffff)} side={DoubleSide} />
	</T.Mesh>
{/each}

{#if $stl && false}
	<T.Mesh geometry={$stl} bind:ref={mesh}>
		<T.MeshNormalMaterial />
	</T.Mesh>
{/if}

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
		Plane,
		Line3,
		Float32BufferAttribute,
		Box3
	} from 'three';
	import { ExtendedTriangle, MeshBVH } from 'three-mesh-bvh';
	import type { Readable } from 'svelte/store';

	export let buildSurface = [300, 300, 300];
	export let layerHeight = 0.2;
	export let nozzleSize = 0.4;
	export let tolerance = 0.005;
	export let progress = 1;

	export let maxNonPlanarAngle = MathUtils.degToRad(20);
	export let bedNormal = new Vector3(0, 0, 1);
	export let extruderNormal = new Vector3(0, 0, -1);
	export let origin = new Vector3(150, 150, 0);

	const stl: Readable<BufferGeometry> = useLoader(STLLoader).load('/benchy.stl');

	let mesh: Mesh;
	let layers: { type: 'line' | 'surface'; geometry: BufferGeometry }[] = [];

	$: if ($stl) {
		const bvh = new MeshBVH($stl);
		const positions = $stl.getAttribute('position');
		const normals = $stl.getAttribute('normal');
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
			// TODO: bottom layers
			if (angle < maxNonPlanarAngle) {
				qualifyingTriangles[i] = true;
				qualifyingTrianglesCount++;
			}
		}
		const includedTriangles = [...qualifyingTriangles];
		const includedTrianglesCount = qualifyingTrianglesCount;

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

		const nonPlanarSurfaces = surfaces.map((surface) => {
			const geometry = new BufferGeometry();
			geometry.setAttribute('position', positions);
			geometry.setAttribute('normal', normals);
			const indices: number[] = Array.from({ length: surface.length * 3 });
			for (let i = 0; i < surface.length; i++) {
				const pos = surface[i] * 3;
				indices[i * 3] = $stl.index!.array[pos];
				indices[i * 3 + 1] = $stl.index!.array[pos + 1];
				indices[i * 3 + 2] = $stl.index!.array[pos + 2];
			}
			geometry.setIndex(indices);
			return new MeshBVH(geometry);
		});
		const activeNonPlanarSurfaces: [number, MeshBVH][] = [];
		const consumedNonPlanarSurfaces = nonPlanarSurfaces.map(() => false);
		const withheldLines: number[][][] = nonPlanarSurfaces.map(() => [[]]);
		const blacklist = Array.from({ length: index.count / 3 }).map(() => false);

		const line = new Line3();
		const targetVector1 = new Vector3();
		const targetVector2 = new Vector3();
		const targetVector3 = new Vector3();
		const layerPlane = new Plane();
		const layerBox = new Box3();
		for (let layer = 0; layer < $stl.boundingBox!.max.z; layer += layerHeight) {
			layerPlane.set(bedNormal, -layer);
			layerBox.set(
				new Vector3(0, 0, layer),
				new Vector3(buildSurface[0], buildSurface[1], layer + layerHeight)
			);
			const layerGeometry = new BufferGeometry();
			const positions: number[] = [];
			for (let i = 0; i < nonPlanarSurfaces.length; i++) {
				if (consumedNonPlanarSurfaces[i]) continue;
				if (layerBox.intersectsBox(nonPlanarSurfaces[i].geometry.boundingBox!)) {
					activeNonPlanarSurfaces.push([i, nonPlanarSurfaces[i]]);
					consumedNonPlanarSurfaces[i] = true;
				}
			}
			for (let i = 0; i < activeNonPlanarSurfaces.length; i++) {
				const [index, surface] = activeNonPlanarSurfaces[i];
				withheldLines[index].push([]);
				if (!layerBox.intersectsBox(surface.geometry.boundingBox!)) {
					activeNonPlanarSurfaces.splice(i, 1);
					i--;
					layers.push({ type: 'surface', geometry: surface.geometry });
					for (const lines of withheldLines[index]) {
						if (lines.length === 0) continue;
						const additionalGeometry = new BufferGeometry();
						additionalGeometry.setAttribute('position', new Float32BufferAttribute(lines, 3));
						layers.push({ type: 'line', geometry: additionalGeometry });
					}
					delete withheldLines[index];
				}
			}

			bvh.shapecast({
				intersectsBounds(box, _isLeaf, _score, _depth, _nodeIndex) {
					return layerPlane.intersectsBox(box);
				},
				intersectsTriangle(target, triangleIndex, _contained, _depth) {
					if (includedTriangles[triangleIndex] || blacklist[triangleIndex]) return;
					function intersect(a: Vector3, b: Vector3, targetVector: Vector3) {
						line.set(a, b);
						return layerPlane.intersectLine(line, targetVector);
					}
					const a = intersect(target.a, target.b, targetVector1);
					const b = intersect(target.b, target.c, targetVector2);
					const c = intersect(target.c, target.a, targetVector3);

					function add(a: Vector3, b: Vector3) {
						for (let i = 0; i < activeNonPlanarSurfaces.length; i++) {
							const [index, surface] = activeNonPlanarSurfaces[i];
							const h1 = surface.closestPointToPoint(a);
							if (
								h1 &&
								h1.point.z < a.z &&
								h1.point.clone().sub(a).angleTo(bedNormal) > maxNonPlanarAngle
							) {
								withheldLines[index].at(-1)!.push(a.x, a.y, a.z, b.x, b.y, b.z);
								return;
							}
							const h2 = surface.closestPointToPoint(b);
							if (
								h2 &&
								h2.point.z < b.z &&
								h2.point.clone().sub(b).angleTo(bedNormal) > maxNonPlanarAngle
							) {
								withheldLines[index].at(-1)!.push(a.x, a.y, a.z, b.x, b.y, b.z);
								return;
							}
						}
						positions.push(a.x, a.y, a.z, b.x, b.y, b.z);
					}

					if (a && b) {
						add(a, b);
					} else if (b && c) {
						add(b, c);
					} else if (c && a) {
						add(c, a);
					}
				}
			});
			layerGeometry.setAttribute('position', new Float32BufferAttribute(positions, 3));
			layers.push({ type: 'line', geometry: layerGeometry });
			console.log('layer', Math.round(layer / layerHeight), positions.length);
		}
		layers = [...layers];
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

{#each layers as { geometry, type }, i}
	{@const visible = progress >= i / layers.length}
	{@const color = new Color(0, i / layers.length, 0.2)}
	{#if type === 'line'}
		<T.LineSegments {geometry} {visible}>
			<T.LineBasicMaterial {color} />
		</T.LineSegments>
	{:else if type === 'surface'}
		<T.Mesh {geometry} {visible}>
			<T.MeshMatcapMaterial {color} side={DoubleSide} />
		</T.Mesh>
	{/if}
{/each}

{#if $stl && false}
	<T.Mesh geometry={$stl} bind:ref={mesh}>
		<T.MeshNormalMaterial />
	</T.Mesh>
{/if}

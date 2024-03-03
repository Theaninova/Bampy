<script lang="ts">
	import { T } from '@threlte/core';
	import { Gizmo, Grid, OrbitControls } from '@threlte/extras';
	import { STLLoader } from 'three/examples/jsm/loaders/STLLoader';
	import { useLoader } from '@threlte/core';
	import {
		BufferGeometry,
		Float32BufferAttribute,
		MathUtils,
		Vector3,
		Mesh,
		Points,
		Triangle,
		DoubleSide,
		Sphere
	} from 'three';
	import { degToRad } from 'three/src/math/MathUtils.js';
	import { MeshBVH } from 'three-mesh-bvh';
	import type { Readable } from 'svelte/store';

	export let buildSurface = [300, 300, 300];
	export let layerHeight = 0.2;
	export let nozzleSize = 0.4;
	export let tolerance = 0.1;

	export let maxNonPlanarAngle = MathUtils.degToRad(20);
	export let bedNormal = new Vector3(0, 0, 1);
	export let origin = new Vector3(150, 150, 0);

	const stl: Readable<BufferGeometry> = useLoader(STLLoader).load('/benchy.stl');

	let mesh: Mesh;
	let surface: BufferGeometry | undefined;

	$: if ($stl) {
		//slice(mesh);
		// we don't really care about the faces, since the vertices bound the area anyways
		// which is the only thing that matters when creating non-planar slices.
		// sort vertices by z, then x, then y in separate index arrays
		// add face index that maps an index/vertex to multiple faces (indices)
		// on each layer get the closest vertex in z, then find the next closest and
		// determine the angle between them. If it's less than the maxNonPlanarAngle
		// add it to the current slice and set it to consumed, if not

		// build bvh for the mesh, query the clostest point.
		// only store the indices for each slice.
		// query the mesh bvh for the closest point while discarding points not
		// in the slice. Keep track of candidates while querying the bvh.

		// need to build bvh live while generating the slices, so angle checks can be done with
		// respect to the closest point in the slice

		const bvh = new MeshBVH($stl);
		const positions = bvh.geometry.getAttribute('position');
		const index = bvh.geometry.index!;

		const qualifyingTriangles = Array.from({ length: index.count / 3 }, () => false);
		let qualifyingTrianglesCount = 0;
		const triangle = new Triangle();
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

		const spheres = Array.from({ length: 3 }, () => new Sphere());
		const vectors = Array.from({ length: 3 }, () => new Vector3());
		const surfaces: number[][] = [];
		while (qualifyingTrianglesCount > 0) {
			const faceIndex = qualifyingTriangles.findIndex((it) => it);
			qualifyingTriangles[faceIndex] = false;
			qualifyingTrianglesCount--;
			const surface = [faceIndex];
			let cursor = 0;
			while (cursor < surface.length) {
				for (let i = 0; i < 3; i++) {
					vectors[i].fromBufferAttribute(positions, index.array[surface[cursor] * 3 + i]);
					spheres[i].set(vectors[i], tolerance);
				}

				bvh.shapecast({
					intersectsBounds(box, _isLeaf, _score, _depth, _nodeIndex) {
						return spheres.some((sphere) => box.intersectsSphere(sphere));
					},
					intersectsTriangle(triangle, triangleIndex, _contained, _depth) {
						if (
							qualifyingTriangles[triangleIndex] &&
							spheres.some((sphere) => triangle.intersectsSphere(sphere))
						) {
							qualifyingTriangles[triangleIndex] = false;
							qualifyingTrianglesCount--;
							surface.push(triangleIndex);
						}
					}
				});

				cursor++;
			}
			surfaces.push(
				surface.flatMap((face) => [
					index.array[face * 3],
					index.array[face * 3 + 1],
					index.array[face * 3 + 2]
				])
			);
		}

		console.log(surfaces);

		surface = new BufferGeometry();
		surface.setAttribute('position', positions);
		surface.setAttribute('normal', $stl.getAttribute('normal'));
		surface.setIndex(surfaces[4]);

		/*const hull: [position: Vector3, index: number][][] = [];
		let limit = 0;
		while (points.length > 0) {
			const consumed = points.map(() => false);
			const currentHull: [position: Vector3, index: number][] = [[points[0][0], 0]];
			consumed[0] = true;
			for (let i = 1; i < points.length; i++) {
				inner: do {
					const b = points[i][0].clone().sub(currentHull[currentHull.length - 1][0]);
					const angle = Math.asin(
						Math.abs(b.clone().dot(bedNormal)) /
							(Math.abs(b.length()) * Math.abs(bedNormal.length()))
					);
					if (angle <= maxNonPlanarAngle) {
						currentHull.push([points[i][0], points[i][2]]);
						consumed[i] = true;
						break inner;
					} else if (points[i][0].z < currentHull[currentHull.length - 1][0].z) {
						consumed[currentHull.pop()![1]] = false;
						if (currentHull.length === 0) {
							currentHull.push([points[i][0], points[i][2]]);
							consumed[i] = true;
							break inner;
						}
					} else {
						break inner;
					}
				} while (true);
			}

			points = points.filter((_, j) => !consumed[j]);
			hull.push(currentHull);
			if (limit++ > 100) break;
		}
		console.log(hull);*/
	}

	async function slice(mesh: Mesh) {
		const { World, ColliderDesc, Ray } = await import('@dimforge/rapier3d');
		console.log(mesh.geometry.getAttribute('indices'));

		const positions = mesh.geometry.getAttribute('position');
		const collider = ColliderDesc.trimesh(
			new Float32Array(positions.array),
			new Uint32Array(
				mesh.geometry.index?.array ?? Array.from({ length: positions.count }, (_, i) => i)
			)
		);
		collider.setTranslation(mesh.position.x, mesh.position.y, mesh.position.z);
		collider.setRotation(mesh.quaternion);
		collider.shape;
		const rayNormal = new Vector3(0, 0, 1);
		const shapePos = new Vector3(0, 0, 0);
		const shapeRot = { x: 0, y: 0, z: 0, w: 0 };
		console.log(((buildSurface[0] / nozzleSize) * buildSurface[1]) / nozzleSize);
		for (let x = 0; x < buildSurface[0]; x += nozzleSize) {
			for (let y = 0; y < buildSurface[1]; y += nozzleSize) {
				// collider.shape.castRay(new Ray({ x, y, z: 0 }, rayNormal), shapePos, shapeRot, 20, false);
			}
		}
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

{#if surface}
	<T.Mesh geometry={surface}>
		<T.MeshNormalMaterial side={DoubleSide} />
	</T.Mesh>
{/if}

{#if $stl && false}
	<T.Mesh geometry={$stl} bind:ref={mesh}>
		<T.MeshNormalMaterial />
	</T.Mesh>
{/if}

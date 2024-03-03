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
		DoubleSide
	} from 'three';
	import { degToRad } from 'three/src/math/MathUtils.js';
	import { MeshBVH } from 'three-mesh-bvh';
	import type { Readable } from 'svelte/store';

	export let buildSurface = [300, 300, 300];
	export let layerHeight = 0.2;
	export let nozzleSize = 0.4;
	export let tolerance = 0.005;

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
		let indices: [number, number, number][] = [];
		const positions = $stl.getAttribute('position');
		console.log($stl.index);

		const triangle = new Triangle();
		const normal = new Vector3();
		for (let i = 0; i < $stl.index!.count; i += 3) {
			triangle.setFromAttributeAndIndices(
				positions,
				$stl.index!.array[i],
				$stl.index!.array[i + 1],
				$stl.index!.array[i + 2]
			);
			triangle.getNormal(normal);
			const angle = normal.angleTo(bedNormal);
			if ((angle > Math.PI / 2 ? Math.PI - angle : angle) < maxNonPlanarAngle) {
				indices.push([$stl.index!.array[i], $stl.index!.array[i + 1], $stl.index!.array[i + 2]]);
			}
		}

		const pointIndex = Array.from({ length: 3 }, (_, j) =>
			Array.from({ length: positions.count }, (_, i) => i).sort(
				(a, b) => positions.array[a * 3 + j] - positions.array[b * 3 + j]
			)
		);
		function findNearby(i: number): number[] {
			const a = [positions.array[i * 3], positions.array[i * 3 + 1], positions.array[i * 3 + 2]];
			const ia = [-1, -1, -1];
			// binary search for the closest points in x, y and z
			for (let j = 0; j < 3; j++) {
				let d = Math.floor(pointIndex[j].length / 2);

				inner: while (d / 2 >= 1) {
					const value = positions.array[pointIndex[j][d] * 3 + j];
					const diff = value - a[j];
					if (Math.abs(diff) < tolerance) {
						ia[j] = d;
						break inner;
					} else if (value < a[j]) {
						d = Math.floor(d / 2);
					} else {
						d = Math.floor(d + d / 2);
					}
				}
				if (ia[j] === -1) return [];
			}
      while ()
		}
    bvh.shapecast({
      intersectsBounds(box, isLeaf, score, depth, nodeIndex) {
        // TODO
      },
        intersectsTriangle(triangle, triangleIndex, contained, depth) {
            // TODO
        }
    })
		const connectedPoints: number[] = Array.from({ length: positions.count });
		let connection = 0;
		for (let i = 0; i < connectedPoints.length; i++) {
			if (connectedPoints[i] !== undefined) continue;
			connectedPoints[i] = i;
			let connected;
			while ((connected = connectedPoints[pointIndex[0][connection]]) !== undefined) {
				connectedPoints[pointIndex[0][connection]] = i;
				connection++;
			}

			connection++;
		}

		const faceConnections = new Map<number, number[]>();
		function spatialHash(i: number) {
			return (
				(positions.getX(i) * 19349663) ^
				(positions.getY(i) * 83492791) ^
				(positions.getZ(i) * 73856093)
			);
		}
		for (let faceIndex = 0; faceIndex < indices.length; faceIndex++) {
			let surface: number[] | undefined = undefined;
			const values = indices[faceIndex].map((i) => {
				const hash = spatialHash(i);
				const value = faceConnections.get(hash);
				surface ??= value;
				return [hash, value] as const;
			});
			surface ??= [];
			surface.push(faceIndex);
			for (const [hash, original] of values) {
				faceConnections.set(hash, surface);
				if (original && original !== surface) {
					surface.concat(original);
				}
			}
		}
		const surfaceSet = new Set(faceConnections.values());
		const iterator = surfaceSet.values();
		const surfaces = Array.from({ length: surfaceSet.size }, () => {
			const value = iterator.next().value;
			return Array.from(
				{ length: value.length * 3 },
				(_, i) => indices[value[Math.floor(i / 3)]][i % 3]
			);
		});

		surface = new BufferGeometry();
		surface.setAttribute('position', positions);
		surface.setAttribute('normal', $stl.getAttribute('normal'));
		surface.setIndex(surfaces[1].flat());

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

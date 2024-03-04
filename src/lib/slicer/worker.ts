import {
	BufferGeometry,
	BufferGeometryLoader,
	Float32BufferAttribute,
	Line3,
	MathUtils,
	Matrix4,
	Plane,
	Vector3
} from 'three';
import { ExtendedTriangle, MeshBVH, type HitPointInfo } from 'three-mesh-bvh';
import {
	LayerType,
	type LayerMessage,
	type SliceArguments,
	type ProgressMessage,
	type WorkerEvent
} from './worker-data';

addEventListener('message', (event: MessageEvent<WorkerEvent>) => {
	if (event.data.type === 'slice') {
		slice(event.data.data);
	}
});

function slice({
	stl,
	bedNormal: bedNormalArray,
	maxNonPlanarAngle,
	tolerance,
	layerHeight
}: SliceArguments) {
	self.postMessage({ type: 'progress', percent: 0, layer: 0 } satisfies ProgressMessage);
	const bedNormal = new Vector3(...bedNormalArray);
	const geometry = new BufferGeometryLoader().parse(stl);
	const bvh = new MeshBVH(geometry);
	const positions = geometry.getAttribute('position');
	const normals = geometry.getAttribute('normal');
	const index = geometry.index!;

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
			indices[i * 3] = index.array[pos];
			indices[i * 3 + 1] = index.array[pos + 1];
			indices[i * 3 + 2] = index.array[pos + 2];
		}
		geometry.setIndex(indices);
		const bvh = new MeshBVH(geometry);
		geometry.boundsTree = bvh;
		return bvh;
	});
	const activeNonPlanarSurfaces: [number, MeshBVH][] = [];
	const consumedNonPlanarSurfaces = nonPlanarSurfaces.map(() => false);
	const withheld: Array<
		| { type: LayerType.Line; geometry: number[] }
		| { type: LayerType.Surface; id: [number, MeshBVH] }
	>[] = nonPlanarSurfaces.map(() => [{ type: LayerType.Line, geometry: [] }]);
	const blacklist = Array.from({ length: index.count / 3 }).map(() => false);

	const line = new Line3();
	const targetVector1 = new Vector3();
	const targetVector2 = new Vector3();
	const targetVector3 = new Vector3();
	const hit1: HitPointInfo = { point: new Vector3(), distance: 0, faceIndex: 0 };
	const hit2: HitPointInfo = { point: new Vector3(), distance: 0, faceIndex: 0 };
	const layerPlane = new Plane();
	function deactivateSurface(surface: MeshBVH, index: number) {
		self.postMessage({
			type: 'layer',
			data: { type: LayerType.Surface, geometry: surface.geometry.toJSON() }
		} satisfies LayerMessage);
		for (const thing of withheld[index]) {
			if (thing.type === LayerType.Line) {
				if (thing.geometry.length === 0) continue;
				const additionalGeometry = new BufferGeometry();
				additionalGeometry.setAttribute('position', new Float32BufferAttribute(thing.geometry, 3));
				self.postMessage({
					type: 'layer',
					data: { type: LayerType.Line, geometry: additionalGeometry.toJSON() }
				});
			} else if (thing.type === LayerType.Surface) {
				deactivateSurface(thing.id[1], thing.id[0]);
			}
		}
		delete withheld[index];
	}
	for (let layer = 0; layer < geometry.boundingBox!.max.z; layer += layerHeight) {
		layerPlane.set(bedNormal, -layer);
		const layerGeometry = new BufferGeometry();
		const positions: number[] = [];
		for (let i = 0; i < nonPlanarSurfaces.length; i++) {
			if (consumedNonPlanarSurfaces[i]) continue;
			if (layer >= nonPlanarSurfaces[i].geometry.boundingBox!.min.z) {
				consumedNonPlanarSurfaces[i] = true;
				activeNonPlanarSurfaces.push([i, nonPlanarSurfaces[i]]);
			}
		}
		deactivate: for (let i = 0; i < activeNonPlanarSurfaces.length; i++) {
			const [index, surface] = activeNonPlanarSurfaces[i];
			if (layer > surface.geometry.boundingBox!.max.z) {
				activeNonPlanarSurfaces.splice(i, 1);
				i--;

				for (const [activeIndex, active] of activeNonPlanarSurfaces) {
					if (activeIndex === index) continue;
					const hit = active.closestPointToGeometry(surface.geometry, new Matrix4(), hit1, hit2);
					if (
						hit &&
						hit1.point.z < hit2.point.z &&
						hit1.point.clone().sub(hit2.point).angleTo(bedNormal) > maxNonPlanarAngle
					) {
						withheld[activeIndex].push({ type: LayerType.Surface, id: [index, surface] });
						withheld[activeIndex].push({ type: LayerType.Line, geometry: [] });
						continue deactivate;
					}
				}
				deactivateSurface(surface, index);
			}
			withheld[index]?.push({ type: LayerType.Line, geometry: [] });
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
						const withheldLayer = withheld[index].at(-1)!;
						if (withheldLayer.type === LayerType.Surface) throw new Error('Unexpected surface');
						const h1 = surface.closestPointToPoint(a);
						if (
							h1 &&
							h1.point.z < a.z &&
							Math.abs(Math.PI / 2 - h1.point.clone().sub(a).angleTo(bedNormal)) > maxNonPlanarAngle
						) {
							withheldLayer.geometry.push(a.x, a.y, a.z, b.x, b.y, b.z);
							return;
						}
						const h2 = surface.closestPointToPoint(b);
						if (
							h2 &&
							h2.point.z < b.z &&
							Math.abs(Math.PI / 2 - h2.point.clone().sub(b).angleTo(bedNormal)) > maxNonPlanarAngle
						) {
							withheldLayer.geometry.push(a.x, a.y, a.z, b.x, b.y, b.z);
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
		self.postMessage({
			type: 'layer',
			data: { type: LayerType.Line, geometry: layerGeometry.toJSON() }
		} satisfies LayerMessage);
		self.postMessage({
			type: 'progress',
			percent: layer / geometry.boundingBox!.max.z,
			layer: Math.round(layer / layerHeight)
		} satisfies ProgressMessage);
	}
	for (const [index, surface] of activeNonPlanarSurfaces) {
		deactivateSurface(surface, index);
	}
	self.postMessage({
		type: 'progress',
		layer: Math.round(geometry.boundingBox!.max.z / layerHeight)
	} satisfies ProgressMessage);
}

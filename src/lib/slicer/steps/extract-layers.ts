import { BufferGeometry, Float32BufferAttribute, Line3, Matrix4, Plane, Vector3 } from 'three';
import type { SlicerOptions } from '../slicer';
import type { HitPointInfo, MeshBVH } from 'three-mesh-bvh';
import { LayerType, type LayerMessage, type ProgressMessage } from '../worker-data';

type Withheld = Array<
	{ type: LayerType.Line; geometry: number[] } | { type: LayerType.Surface; id: [number, MeshBVH] }
>[];

function deactivateSurface(this: Withheld, surface: MeshBVH, index: number) {
	self.postMessage({
		type: 'layer',
		data: { type: LayerType.Surface, geometry: surface.geometry.toJSON() }
	} satisfies LayerMessage);

	for (const thing of this[index]) {
		if (thing.type === LayerType.Line) {
			if (thing.geometry.length === 0) continue;
			const additionalGeometry = new BufferGeometry();
			additionalGeometry.setAttribute('position', new Float32BufferAttribute(thing.geometry, 3));
			self.postMessage({
				type: 'layer',
				data: { type: LayerType.Line, geometry: additionalGeometry.toJSON() }
			});
		} else if (thing.type === LayerType.Surface) {
			deactivateSurface.call(this, thing.id[1], thing.id[0]);
		}
	}
	delete this[index];
}

const line = new Line3();
function intersect(layerPlane: Plane, a: Vector3, b: Vector3, targetVector: Vector3) {
	line.set(a, b);
	return layerPlane.intersectLine(line, targetVector);
}

export function extractLayers(
	options: SlicerOptions,
	surfaces: MeshBVH[],
	surfaceTriangles: boolean[]
) {
	const targetVector1 = new Vector3();
	const targetVector2 = new Vector3();
	const targetVector3 = new Vector3();
	const hit1: HitPointInfo = { point: new Vector3(), distance: 0, faceIndex: 0 };
	const hit2: HitPointInfo = { point: new Vector3(), distance: 0, faceIndex: 0 };
	const layerPlane = new Plane();

	const activeNonPlanarSurfaces: [number, MeshBVH][] = [];
	const consumedNonPlanarSurfaces = surfaces.map(() => false);
	const withheld: Withheld = surfaces.map(() => [{ type: LayerType.Line, geometry: [] }]);
	const blacklist = Array.from({ length: options.index.count / 3 }).map(() => false);

	for (let layer = 0; layer < options.geometry.boundingBox!.max.z; layer += options.layerHeight) {
		layerPlane.set(options.bedNormal, -layer);
		const layerGeometry = new BufferGeometry();
		const positions: number[] = [];
		for (let i = 0; i < surfaces.length; i++) {
			if (consumedNonPlanarSurfaces[i]) continue;
			if (layer >= surfaces[i].geometry.boundingBox!.min.z) {
				consumedNonPlanarSurfaces[i] = true;
				activeNonPlanarSurfaces.push([i, surfaces[i]]);
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
						Math.abs(Math.PI / 2 - hit1.point.clone().sub(hit2.point).angleTo(options.bedNormal)) >
							options.maxNonPlanarAngle
					) {
						withheld[activeIndex].push({ type: LayerType.Surface, id: [index, surface] });
						withheld[activeIndex].push({ type: LayerType.Line, geometry: [] });
						continue deactivate;
					}
				}
				deactivateSurface.call(withheld, surface, index);
			}
			withheld[index]?.push({ type: LayerType.Line, geometry: [] });
		}

		options.bvh.shapecast({
			intersectsBounds(box, _isLeaf, _score, _depth, _nodeIndex) {
				return layerPlane.intersectsBox(box);
			},
			intersectsTriangle(target, triangleIndex, _contained, _depth) {
				if (surfaceTriangles[triangleIndex] || blacklist[triangleIndex]) return;
				const targets = [target.a, target.b, target.c];
				const items = [targetVector1, targetVector2, targetVector3];

				let a: Vector3 | null = intersect(layerPlane, targets[0], targets[1], targetVector1);
				let b: Vector3 | null = null;
				for (let i = 0; i < 3; i++) {
					const i1 = (i + 1) % 3;
					a = b;
					b = intersect(
						layerPlane,
						targets[i1],
						targets[(i1 + 1) % 3],
						i % 2 === 0 ? targetVector2 : targetVector1
					);
					if (!a || !b) continue;

					for (let i = 0; i < activeNonPlanarSurfaces.length; i++) {
						const [index, surface] = activeNonPlanarSurfaces[i];
						const withheldLayer = withheld[index].at(-1)!;
						if (withheldLayer.type === LayerType.Surface) throw new Error('Unexpected surface');
						const h1 = surface.closestPointToPoint(a);
						if (
							h1 &&
							h1.point.z < a.z &&
							Math.abs(Math.PI / 2 - h1.point.clone().sub(a).angleTo(options.bedNormal)) >
								options.maxNonPlanarAngle
						) {
							withheldLayer.geometry.push(a.x, a.y, a.z, b.x, b.y, b.z);
							return;
						}
						const h2 = surface.closestPointToPoint(b);
						if (
							h2 &&
							h2.point.z < b.z &&
							Math.abs(Math.PI / 2 - h2.point.clone().sub(b).angleTo(options.bedNormal)) >
								options.maxNonPlanarAngle
						) {
							withheldLayer.geometry.push(a.x, a.y, a.z, b.x, b.y, b.z);
							return;
						}
					}
					positions.push(a.x, a.y, a.z, b.x, b.y, b.z);
					return;
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
			percent: layer / options.geometry.boundingBox!.max.z,
			layer: Math.round(layer / options.layerHeight)
		} satisfies ProgressMessage);
	}

	for (const [index, surface] of activeNonPlanarSurfaces) {
		deactivateSurface.call(withheld, surface, index);
	}
}

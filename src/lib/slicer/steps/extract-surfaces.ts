import { Vector3, BufferGeometry } from 'three';
import { type SlicerOptions } from '../slicer';
import { ExtendedTriangle, MeshBVH } from 'three-mesh-bvh';

/**
 * Extracts all continuous surfaces that can be printed at the specified angle.
 */
export function extractSurfaces(
	options: SlicerOptions
): [surfaces: MeshBVH[], surfaceTriangles: boolean[]] {
	const qualifyingTriangles = Array.from({ length: options.index.count / 3 }, () => false);
	let qualifyingTrianglesCount = 0;
	const triangle = new ExtendedTriangle();
	const normal = new Vector3();
	for (let i = 0; i < options.index.count / 3; i++) {
		triangle.setFromAttributeAndIndices(
			options.positions,
			options.index.array[i * 3],
			options.index.array[i * 3 + 1],
			options.index.array[i * 3 + 2]
		);
		triangle.getNormal(normal);
		const angle = normal.angleTo(options.bedNormal);
		// TODO: bottom layers
		if (angle < options.maxNonPlanarAngle) {
			qualifyingTriangles[i] = true;
			qualifyingTrianglesCount++;
		}
	}

	const surfaceTriangles = [...qualifyingTriangles];

	const surfaces: number[][] = [];
	while (qualifyingTrianglesCount > 0) {
		const faceIndex = qualifyingTriangles.findIndex((it) => it);
		qualifyingTriangles[faceIndex] = false;
		qualifyingTrianglesCount--;
		const surface = [faceIndex];
		let cursor = 0;
		while (cursor < surface.length) {
			triangle.setFromAttributeAndIndices(
				options.positions,
				options.index.array[surface[cursor] * 3],
				options.index.array[surface[cursor] * 3 + 1],
				options.index.array[surface[cursor] * 3 + 2]
			);

			options.bvh.shapecast({
				intersectsBounds(box, _isLeaf, _score, _depth, _nodeIndex) {
					return triangle.intersectsBox(box);
				},
				intersectsTriangle(target, triangleIndex, _contained, _depth) {
					if (
						qualifyingTriangles[triangleIndex] &&
						target.distanceToTriangle(triangle) < options.tolerance
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

	return [
		surfaces.map((surface) => {
			const geometry = new BufferGeometry();
			geometry.setAttribute('position', options.positions);
			geometry.setAttribute('normal', options.normals);
			const indices: number[] = Array.from({ length: surface.length * 3 });
			for (let i = 0; i < surface.length; i++) {
				const pos = surface[i] * 3;
				indices[i * 3] = options.index.array[pos];
				indices[i * 3 + 1] = options.index.array[pos + 1];
				indices[i * 3 + 2] = options.index.array[pos + 2];
			}
			geometry.setIndex(indices);
			const bvh = new MeshBVH(geometry);
			geometry.boundsTree = bvh;
			return bvh;
		}),
		surfaceTriangles
	];
}

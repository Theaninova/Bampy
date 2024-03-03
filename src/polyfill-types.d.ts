import { MeshBVH, computeBoundsTree, disposeBoundsTree } from 'three-mesh-bvh';

declare module 'three' {
	export interface BufferGeometry {
		boundsTree?: MeshBVH;
		computeBoundsTree: typeof computeBoundsTree;
		disposeBoundsTree: typeof disposeBoundsTree;
	}
}

export {};

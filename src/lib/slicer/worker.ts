import { BufferGeometry, BufferGeometryLoader, Float32BufferAttribute } from 'three';
import {
	type SliceArguments,
	type ProgressMessage,
	type WorkerEvent,
	type LayerMessage,
	LayerType
} from './worker-data';
import init, { slice } from 'bampy';

addEventListener('message', async (event: MessageEvent<WorkerEvent>) => {
	if (event.data.type === 'slice') {
		const geometry = new BufferGeometryLoader().parse(event.data.data.stl);
		if (geometry.index !== null) {
			geometry.toNonIndexed();
		}
		await init();
		const result = slice({
			positions: geometry.attributes.position.array as Float32Array,
			layerHeight: event.data.data.layerHeight,
			maxAngle: event.data.data.maxNonPlanarAngle
		});
		for (const layer of result.slices) {
			const geometry = new BufferGeometry();
			geometry.setAttribute('position', new Float32BufferAttribute(layer.position, 3));
			if (layer.type === 'surface') {
				geometry.computeVertexNormals();
			}

			self.postMessage({
				type: 'layer',
				data: {
					type: layer.type === 'ring' ? LayerType.Line : LayerType.Surface,
					geometry: geometry.toJSON()
				}
			} satisfies LayerMessage);
		}
	}
});

async function todo({
	stl,
	bedNormal: bedNormalArray,
	maxNonPlanarAngle,
	tolerance,
	layerHeight
}: SliceArguments) {
	greet();
	self.postMessage({ type: 'progress', percent: 0, layer: 0 } satisfies ProgressMessage);

	// TODO

	self.postMessage({
		type: 'progress',
		layer: Math.round(geometry.boundingBox!.max.z / layerHeight)
	} satisfies ProgressMessage);
}

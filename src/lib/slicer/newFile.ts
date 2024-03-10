import { BufferGeometry, BufferGeometryLoader, Float32BufferAttribute } from 'three';
import { type WorkerEvent } from './worker-data';
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
		for (const layer of result.rings) {
			const geometry = new BufferGeometry();
			geometry.setAttribute('position', new Float32BufferAttribute(layer, 3));

			self.postMessage({
				type: 'layer',
				data: { type: LayerType.Line, geometry: layerGeometry.toJSON() }
			} satisfies LayerMessage);
		}
	}
});

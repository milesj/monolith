import cytoscape from 'cytoscape';
import dagre from 'cytoscape-dagre';
import type { GraphInfo } from './types';

cytoscape.use(dagre);

function getActionType(label: string) {
	if (label.startsWith('RunTarget')) {
		return 'run-target';
	}

	if (label.startsWith('Sync') && label.includes('Project')) {
		return 'sync-project';
	}

	if (label.startsWith('Install') && label.includes('Deps')) {
		return 'install-deps';
	}

	if (label.startsWith('Setup') && label.includes('Tool')) {
		return 'setup-tool';
	}

	return 'unknown';
}

export function render(element: HTMLElement, data: GraphInfo) {
	const nodes = data.nodes.map((n) => ({
		data: { id: n.id.toString(), label: n.label, type: getActionType(n.label) },
	}));

	const edges = data.edges.map((e) => ({
		data: { id: e.id.toString(), source: e.source.toString(), target: e.target.toString() },
	}));

	// https://js.cytoscape.org/
	return cytoscape({
		container: element,
		elements: { edges, nodes },
		layout: {
			fit: true,
			// @ts-expect-error Types incorrect
			name: 'dagre',
			nodeDimensionsIncludeLabels: true,
			spacingFactor: 1.5,
		},
		style: [
			{
				selector: 'edges',
				style: {
					'arrow-scale': 2,
					'curve-style': 'straight',
					'line-cap': 'round',
					'line-color': '#c9eef6', // '#012a4a',
					'line-opacity': 0.15,
					'overlay-color': '#c9eef6',
					'target-arrow-color': '#c9eef6', // '#1a3f5c',
					'target-arrow-shape': 'tee',
					width: 3,
				},
			},
			{
				selector: 'node',
				style: {
					// @ts-expect-error Types incorrect
					'background-fill': 'linear-gradient',
					'background-gradient-direction': 'to-bottom-right',
					'background-gradient-stop-colors': '#d7dfe9 #bdc9db #97a1af',
					color: '#fff',
					height: 60,
					label: 'data(label)',
					'overlay-color': '#99aab7',
					'overlay-shape': 'ellipse',
					padding: 0,
					shape: 'ellipse',
					'text-halign': 'center',
					'text-margin-y': 6,
					'text-valign': 'bottom',
					'underlay-shape': 'ellipse',
					width: 60,
				},
			},
			{
				selector: 'node[type="run-target"]',
				style: {
					// @ts-expect-error Types incorrect
					'background-gradient-stop-colors': '#6e58d1 #4a2ec6 #3b259e',
				},
			},
			{
				selector: 'node[type="sync-project"]',
				style: {
					// @ts-expect-error Types incorrect
					'background-gradient-stop-colors': '#ffafff #ff79ff #cc61cc',
					height: 80,
					width: 80,
				},
			},
			{
				selector: 'node[type="install-deps"]',
				style: {
					// @ts-expect-error Types incorrect
					'background-gradient-stop-colors': '#afe6f2 #79d5e9 #61aaba',
					height: 100,
					width: 100,
				},
			},
			{
				selector: 'node[type="setup-tool"]',
				style: {
					// @ts-expect-error Types incorrect
					'background-gradient-stop-colors': '#ff9da6 #ff5b6b #cc4956',
					height: 120,
					width: 120,
				},
			},
		],
	});
}

import React, { useEffect, useRef } from 'react';

import G6 from '@antv/g6';
import testData from '../test-data.ts';

function Graph() {
  const ref = useRef(null);

  let graph = null;

  useEffect(
    () => {
      if (!graph) {
        graph = new G6.Graph({
          container: ref.current,
          width: 1200,
          height: 800,
          modes: {
            default: ['drag-canvas', 'zoom-canvas', 'drag-node'],
          },
          layout: { type: 'dagre' },
          defaultNode: {
            style: {
              r: 20,
            },
          },
          defaultEdge: {
            style: {
              endArrow: true,
            },
          },
          nodeStateStyles: {
            hover: {
              fill: 'lightsteelblue',
            },
            highlight: {
              stroke: '#000',
              lineWidth: 3,
            },
            lowlight: {
              opacity: 0.3,
            },
          },
          edgeStateStyles: {
            lowlight: {
              opacity: 0.3,
            },
          },
        });
      }

      // Mouse enter a node
      graph.on('node:mouseenter', (e) => {
        const nodeItem = e.item; // Get the target item
        graph.setItemState(nodeItem, 'hover', true); // Set the state 'hover' of the item to be true
      });

      // Mouse leave a node
      graph.on('node:mouseleave', (e) => {
        const nodeItem = e.item; // Get the target item
        graph.setItemState(nodeItem, 'hover', false); // Set the state 'hover' of the item to be false
      });

      // Click a node
      graph.on('node:click', (e) => {
        // Swich the 'click' state of the node to be false
        const clickNodes = graph.findAllByState('node', 'highlight');
        clickNodes.forEach((cn) => {
          graph.setItemState(cn, 'highlight', false);
        });

        graph.getNodes().forEach((node) => {
          graph.setItemState(node, 'lowlight', true);
        });
        graph.getEdges().forEach((edge) => {
          graph.setItemState(edge, 'lowlight', true);
        });

        const nodeItem = e.item; // et the clicked item
        graph.setItemState(nodeItem, 'lowlight', false);
        graph.setItemState(nodeItem, 'highlight', true);
        nodeItem.getEdges().forEach((edge) => {
          graph.setItemState(edge, 'lowlight', false);
        });
      });

      graph.data({
        nodes: testData.map((node, index) => ({ id: index.toString(), label: node.label })),
        edges: testData.flatMap((node, index) => [{ source: index.toString(), target: node.lo.toString(), style: { stroke: 'red', lineWidth: 2 } }, { source: index.toString(), target: node.hi.toString(), style: { stroke: 'green', lineWidth: 2 } }]),
      });
      graph.render();
    },
    [],
  );

  return <div ref={ref} />;
}

export default Graph;

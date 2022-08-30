import React, { useEffect, useRef } from 'react';

import G6 from '@antv/g6';

interface Props {
  graph: {
    lo_edges: [number, number][],
    hi_edges: [number, number][],
    node_labels: { [key: number]: string },
    tree_root_labels: { [key: number]: string[] },
  }
}

function Graph(props: Props) {
  const { graph: graphProps } = props;

  const ref = useRef(null);

  const graphRef = useRef();

  useEffect(
    () => {
      if (!graphRef.current) {
        graphRef.current = new G6.Graph({
          container: ref.current,
          width: 1200,
          height: 600,
          fitView: true,
          rankdir: 'TB',
          align: 'DR',
          nodesep: 100,
          ranksep: 100,
          modes: {
            default: ['drag-canvas', 'zoom-canvas', 'drag-node'],
          },
          layout: { type: 'dagre' },
          defaultNode: {
            anchorPoints: [[0.5, 0], [0, 0.5], [1, 0.5], [0.5, 1]],
            type: 'rect',
            style: {
              radius: 2,
            },
            labelCfg: {
              style: {
                // fontWeight: 700,
                fontFamily: 'Roboto',
              },
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

      const graph = graphRef.current;

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
        const nodeItem = e.item; // et the clicked item

        let onlyRemoveStates = false;
        if (nodeItem.hasState('highlight')) {
          onlyRemoveStates = true;
        }

        const clickNodes = graph.findAllByState('node', 'highlight');
        clickNodes.forEach((cn) => {
          graph.setItemState(cn, 'highlight', false);
        });

        const lowlightNodes = graph.findAllByState('node', 'lowlight');
        lowlightNodes.forEach((cn) => {
          graph.setItemState(cn, 'lowlight', false);
        });
        const lowlightEdges = graph.findAllByState('edge', 'lowlight');
        lowlightEdges.forEach((cn) => {
          graph.setItemState(cn, 'lowlight', false);
        });

        if (onlyRemoveStates) {
          return;
        }

        graph.getNodes().forEach((node) => {
          graph.setItemState(node, 'lowlight', true);
        });
        graph.getEdges().forEach((edge) => {
          graph.setItemState(edge, 'lowlight', true);
        });

        const relevantNodeIds = [];
        const relevantLoEdges = [];
        const relevantHiEdges = [];
        let newNodeIds = [nodeItem.getModel().id];
        let newLoEdges = [];
        let newHiEdges = [];

        while (newNodeIds.length > 0 || newLoEdges.length > 0 || newHiEdges.length > 0) {
          relevantNodeIds.push(...newNodeIds);
          relevantLoEdges.push(...newLoEdges);
          relevantHiEdges.push(...newHiEdges);

          newLoEdges = graphProps.lo_edges
            .filter((edge) => relevantNodeIds.includes(edge[0].toString())
              && !relevantLoEdges.includes(edge));
          newHiEdges = graphProps.hi_edges
            .filter((edge) => relevantNodeIds.includes(edge[0].toString())
              && !relevantHiEdges.includes(edge));

          newNodeIds = newLoEdges
            .concat(newHiEdges)
            .map((edge) => edge[1].toString())
            .filter((id) => !relevantNodeIds.includes(id));
        }

        const relevantEdgeIds = relevantLoEdges
          .map(([source, target]) => `LO_${source}_${target}`)
          .concat(
            relevantHiEdges
              .map(([source, target]) => `HI_${source}_${target}`),
          );

        console.log(relevantNodeIds);
        console.log(relevantEdgeIds);

        relevantNodeIds
          .forEach((id) => {
            graph.setItemState(id, 'lowlight', false);
            graph.setItemState(id, 'highlight', true);
          });

        relevantEdgeIds
          .forEach((id) => {
            graph.setItemState(id, 'lowlight', false);
          });

        // graph.setItemState(nodeItem, 'lowlight', false);
        // graph.setItemState(nodeItem, 'highlight', true);
        // nodeItem.getEdges().forEach((edge) => {
        // graph.setItemState(edge, 'lowlight', false);
        // });
      });
    },
    [],
  );

  useEffect(
    () => {
      const graph = graphRef.current;

      const nodes = Object.keys(graphProps.node_labels).map((id) => {
        const mainLabel = graphProps.node_labels[id];
        const subLabel = graphProps.tree_root_labels[id].length > 0 ? `Root for: ${graphProps.tree_root_labels[id].join(' ; ')}` : '';

        const label = subLabel.length > 0 ? `${mainLabel}\n${subLabel}` : mainLabel;

        return {
          id: id.toString(),
          label,
          style: {
            height: subLabel.length > 0 ? 60 : 30,
            width: Math.max(30, 10 * mainLabel.length, 10 * subLabel.length),
          },
        };
      });
      const edges = graphProps.lo_edges.map(([source, target]) => ({
        id: `LO_${source}_${target}`, source: source.toString(), target: target.toString(), style: { stroke: 'red', lineWidth: 2 },
      }))
        .concat(graphProps.hi_edges.map(([source, target]) => ({
          id: `HI_${source}_${target}`, source: source.toString(), target: target.toString(), style: { stroke: 'green', lineWidth: 2 },
        })));

      graph.data({
        nodes,
        edges,
      });
      graph.render();
    },
    [graphProps],
  );

  return <div ref={ref} style={{ overflow: 'hidden' }} />;
}

export default Graph;

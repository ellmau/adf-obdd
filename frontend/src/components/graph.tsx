import React, { useEffect, useRef } from 'react';

import G6 from '@antv/g6';

G6.registerNode('nodeWithFlag', {
  draw(cfg, group) {
    console.log('cfg', cfg);

    const mainWidth = Math.max(30, 5 * cfg.mainLabel.length + 10);
    const mainHeight = 30;

    const keyShape = group.addShape('rect', {
      attrs: {
        width: mainWidth,
        height: mainHeight,
        radius: 2,
        fill: cfg.fill || 'white',
        stroke: 'black',
        lineWidth: cfg.lineWidth,
        opacity: cfg.opacity,
      },
      name: 'rectMainLabel',
      draggable: true,
    });

    group.addShape('text', {
      attrs: {
        x: mainWidth / 2,
        y: mainHeight / 2,
        textAlign: 'center',
        textBaseline: 'middle',
        text: cfg.mainLabel,
        fill: '#212121',
        fontFamily: 'Roboto',
      },
      // must be assigned in G6 3.3 and later versions. it can be any value you want
      name: 'textMailLabel',
      // allow the shape to response the drag events
      draggable: true,
    });

    if (cfg.subLabel) {
      const subWidth = 5 * cfg.subLabel.length + 4;
      const subHeight = 20;

      const subRectX = mainWidth - 4;
      const subRectY = -subHeight + 4;

      group.addShape('rect', {
        attrs: {
          x: subRectX,
          y: subRectY,
          width: subWidth,
          height: subHeight,
          radius: 1,
          fill: '#4caf50',
          stroke: '#1b5e20',
          opacity: cfg.opacity,
        },
        name: 'rectMainLabel',
        draggable: true,
      });

      group.addShape('text', {
        attrs: {
          x: subRectX + subWidth / 2,
          y: subRectY + subHeight / 2,
          textAlign: 'center',
          textBaseline: 'middle',
          text: cfg.subLabel,
          fill: '#212121',
          fontFamily: 'Roboto',
          fontSize: 10,
        },
        // must be assigned in G6 3.3 and later versions. it can be any value you want
        name: 'textMailLabel',
        // allow the shape to response the drag events
        draggable: true,
      });
    }

    return keyShape;
  },
  getAnchorPoints() {
    return [[0.5, 0], [0, 0.5], [1, 0.5], [0.5, 1]];
  },
  // nodeStateStyles: {
  // hover: {
  // fill: 'lightsteelblue',
  // },
  // highlight: {
  // lineWidth: 3,
  // },
  // lowlight: {
  // opacity: 0.3,
  // },
  // },
  setState(name, value, item) {
    const group = item.getContainer();
    const shape = group.get('children')[0]; // Find the first graphics shape of the node. It is determined by the order of being added

    if (name === 'hover') {
      if (value) {
        shape.attr('fill', 'lightsteelblue');
      } else {
        shape.attr('fill', 'white');
      }
    }

    if (name === 'highlight') {
      if (value) {
        shape.attr('lineWidth', 3);
      } else {
        shape.attr('lineWidth', 1);
      }
    }

    if (name === 'lowlight') {
      if (value) {
        shape.attr('opacity', 0.3);
      } else {
        shape.attr('opacity', 1);
      }
    }
  },
});

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
          // defaultNode: {
          // anchorPoints: [[0.5, 0], [0, 0.5], [1, 0.5], [0.5, 1]],
          // type: 'rect',
          // style: {
          // radius: 2,
          // },
          // labelCfg: {
          // style: {
          /// / fontWeight: 700,
          // fontFamily: 'Roboto',
          // },
          // },
          // },
          defaultNode: { type: 'nodeWithFlag' },
          defaultEdge: {
            style: {
              endArrow: true,
            },
          },
          // nodeStateStyles: {
          // hover: {
          // fill: 'lightsteelblue',
          // },
          // highlight: {
          // lineWidth: 3,
          // },
          // lowlight: {
          // opacity: 0.3,
          // },
          // },
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
    },
    [],
  );

  useEffect(
    () => {
      const graph = graphRef.current;

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

      return () => { graph.off('node:click'); };
    },
    [graphProps],
  );

  useEffect(
    () => {
      const graph = graphRef.current;

      const nodes = Object.keys(graphProps.node_labels).map((id) => {
        const mainLabel = graphProps.node_labels[id];
        const subLabel = graphProps.tree_root_labels[id].length > 0 ? `Root for: ${graphProps.tree_root_labels[id].join(' ; ')}` : undefined;

        // const label = subLabel.length > 0 ? `${mainLabel}\n${subLabel}` : mainLabel;

        return {
          id: id.toString(),
          mainLabel,
          subLabel,
          // style: {
          // height: subLabel.length > 0 ? 60 : 30,
          // width: Math.max(30, 5 * mainLabel.length + 10, 5 * subLabel.length + 10),
          // },
        };
      });
      const edges = graphProps.lo_edges.map(([source, target]) => ({
        id: `LO_${source}_${target}`, source: source.toString(), target: target.toString(), style: { stroke: '#ed6c02', lineWidth: 2 },
      }))
        .concat(graphProps.hi_edges.map(([source, target]) => ({
          id: `HI_${source}_${target}`, source: source.toString(), target: target.toString(), style: { stroke: '#1976d2', lineWidth: 2 },
        })));

      graph.data({
        nodes,
        edges,
      });
      graph.render();
    },
    [graphProps],
  );

  return (
    <>
      <div ref={ref} style={{ overflow: 'hidden' }} />
      <div>
        <span style={{ color: '#ed6c02' }}>lo edge (condition is false)</span>
        {' '}
        <span style={{ color: '#1976d2' }}>hi edge (condition is true)</span>
      </div>
    </>
  );
}

export default Graph;

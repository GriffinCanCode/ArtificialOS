/**
 * Graph Component
 * Renders network graphs for AI thought visualization
 */

import React, { useCallback } from "react";
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
} from "reactflow";
import "reactflow/dist/style.css";
import type { BaseComponentProps } from "../../dynamics/core/types";
import { useSyncState } from "../../dynamics/hooks/useSyncState";
import { getReactFlowTheme } from "../utils";
import type { GraphProps } from "../types";

export const Graph: React.FC<BaseComponentProps> = ({ component, state }) => {
  const props = component.props as GraphProps;

  // Subscribe to data changes
  const graphNodes = useSyncState(state, `${component.id}.nodes`, props.nodes || []);
  const graphEdges = useSyncState(state, `${component.id}.edges`, props.edges || []);

  // Convert to ReactFlow format
  const initialNodes: Node[] = graphNodes.map((n) => ({
    id: n.id,
    type: n.type || "default",
    data: { label: n.label || n.id, ...n.data },
    position: n.position || { x: 0, y: 0 },
    style: n.style,
  }));

  const initialEdges: Edge[] = graphEdges.map((e) => ({
    id: e.id,
    source: e.source,
    target: e.target,
    type: e.type || "default",
    label: e.label,
    animated: e.animated,
    style: e.style,
  }));

  const [nodes, , onNodesChange] = useNodesState(initialNodes);
  const [edges, , onEdgesChange] = useEdgesState(initialEdges);

  // Theme
  const theme = props.theme || "dark";
  const flowTheme = getReactFlowTheme(theme);

  // Dimensions
  const width = props.dimensions?.width;
  const height = props.dimensions?.height || 600;

  // Controls
  const controls = props.controls || {};
  const showControls = controls.zoom !== false;
  const showMinimap = controls.minimap === true;
  const showBackground = controls.background !== false;
  const fitView = controls.fitView !== false;

  const onConnect = useCallback((params: any) => {
    // Handle connection if needed
    console.log("Connection:", params);
  }, []);

  return (
    <div
      className="graph-container"
      style={{
        width: width || "100%",
        height,
        background: flowTheme.background,
      }}
    >
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        fitView={fitView}
        nodesDraggable={props.interactive !== false}
        nodesConnectable={props.interactive !== false}
        elementsSelectable={props.interactive !== false}
        style={{ background: flowTheme.background }}
      >
        {showBackground && (
          <Background color={flowTheme.edge.stroke} gap={16} />
        )}

        {showControls && <Controls />}

        {showMinimap && (
          <MiniMap
            style={{ background: flowTheme.minimap.background }}
            nodeColor={flowTheme.node.background}
          />
        )}
      </ReactFlow>
    </div>
  );
};

Graph.displayName = "Graph";

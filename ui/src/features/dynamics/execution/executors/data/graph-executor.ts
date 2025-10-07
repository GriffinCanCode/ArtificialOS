/**
 * Graph Tool Executor
 * Handles network graph operations for AI thought visualization
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";
import type { GraphNode, GraphEdge } from "../../../../visualization/types";

export class GraphExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    switch (action) {
      case "updateNodes":
        return this.updateNodes(params);

      case "updateEdges":
        return this.updateEdges(params);

      case "addNode":
        return this.addNode(params);

      case "addEdge":
        return this.addEdge(params);

      case "removeNode":
        return this.removeNode(params);

      case "removeEdge":
        return this.removeEdge(params);

      case "highlightNode":
        return this.highlightNode(params);

      case "highlightPath":
        return this.highlightPath(params);

      case "layout":
        return this.applyLayout(params);

      case "clear":
        return this.clear(params);

      default:
        logger.warn("Unknown graph action", { component: "GraphExecutor", action });
        return null;
    }
  }

  /**
   * Update all nodes
   */
  private updateNodes(params: Record<string, any>): void {
    const { graphId, nodes } = params;
    if (!graphId || !nodes) return;

    this.context.componentState.set(`${graphId}.nodes`, nodes);
    logger.debug("Graph nodes updated", {
      component: "GraphExecutor",
      graphId,
      nodeCount: nodes.length,
    });
  }

  /**
   * Update all edges
   */
  private updateEdges(params: Record<string, any>): void {
    const { graphId, edges } = params;
    if (!graphId || !edges) return;

    this.context.componentState.set(`${graphId}.edges`, edges);
    logger.debug("Graph edges updated", {
      component: "GraphExecutor",
      graphId,
      edgeCount: edges.length,
    });
  }

  /**
   * Add single node
   */
  private addNode(params: Record<string, any>): void {
    const { graphId, node } = params;
    if (!graphId || !node) return;

    const currentNodes = this.context.componentState.get(`${graphId}.nodes`) || [];
    const newNodes = [...currentNodes, node];

    this.context.componentState.set(`${graphId}.nodes`, newNodes);
    logger.debug("Graph node added", {
      component: "GraphExecutor",
      graphId,
      nodeId: node.id,
      totalNodes: newNodes.length,
    });
  }

  /**
   * Add single edge
   */
  private addEdge(params: Record<string, any>): void {
    const { graphId, edge } = params;
    if (!graphId || !edge) return;

    const currentEdges = this.context.componentState.get(`${graphId}.edges`) || [];
    const newEdges = [...currentEdges, edge];

    this.context.componentState.set(`${graphId}.edges`, newEdges);
    logger.debug("Graph edge added", {
      component: "GraphExecutor",
      graphId,
      edgeId: edge.id,
      totalEdges: newEdges.length,
    });
  }

  /**
   * Remove node by ID
   */
  private removeNode(params: Record<string, any>): void {
    const { graphId, nodeId } = params;
    if (!graphId || !nodeId) return;

    const currentNodes: GraphNode[] = this.context.componentState.get(`${graphId}.nodes`) || [];
    const currentEdges: GraphEdge[] = this.context.componentState.get(`${graphId}.edges`) || [];

    // Remove node
    const newNodes = currentNodes.filter((n) => n.id !== nodeId);

    // Remove connected edges
    const newEdges = currentEdges.filter((e) => e.source !== nodeId && e.target !== nodeId);

    this.context.componentState.set(`${graphId}.nodes`, newNodes);
    this.context.componentState.set(`${graphId}.edges`, newEdges);

    logger.debug("Graph node removed", {
      component: "GraphExecutor",
      graphId,
      nodeId,
      remainingNodes: newNodes.length,
      removedEdges: currentEdges.length - newEdges.length,
    });
  }

  /**
   * Remove edge by ID
   */
  private removeEdge(params: Record<string, any>): void {
    const { graphId, edgeId } = params;
    if (!graphId || !edgeId) return;

    const currentEdges: GraphEdge[] = this.context.componentState.get(`${graphId}.edges`) || [];
    const newEdges = currentEdges.filter((e) => e.id !== edgeId);

    this.context.componentState.set(`${graphId}.edges`, newEdges);
    logger.debug("Graph edge removed", {
      component: "GraphExecutor",
      graphId,
      edgeId,
      remainingEdges: newEdges.length,
    });
  }

  /**
   * Highlight specific node
   */
  private highlightNode(params: Record<string, any>): void {
    const { graphId, nodeId, style } = params;
    if (!graphId || !nodeId) return;

    const currentNodes: GraphNode[] = this.context.componentState.get(`${graphId}.nodes`) || [];
    const newNodes = currentNodes.map((n) =>
      n.id === nodeId ? { ...n, style: { ...n.style, ...style, border: "2px solid #667eea" } } : n
    );

    this.context.componentState.set(`${graphId}.nodes`, newNodes);
    logger.debug("Node highlighted", {
      component: "GraphExecutor",
      graphId,
      nodeId,
    });
  }

  /**
   * Highlight path between nodes
   */
  private highlightPath(params: Record<string, any>): void {
    const { graphId, path } = params;
    if (!graphId || !path || path.length < 2) return;

    const currentEdges: GraphEdge[] = this.context.componentState.get(`${graphId}.edges`) || [];

    // Create set of edges in path
    const pathEdges = new Set<string>();
    for (let i = 0; i < path.length - 1; i++) {
      pathEdges.add(`${path[i]}-${path[i + 1]}`);
    }

    const newEdges = currentEdges.map((e) => {
      const key = `${e.source}-${e.target}`;
      return pathEdges.has(key)
        ? { ...e, style: { ...e.style, stroke: "#667eea", strokeWidth: 3 }, animated: true }
        : e;
    });

    this.context.componentState.set(`${graphId}.edges`, newEdges);
    logger.debug("Path highlighted", {
      component: "GraphExecutor",
      graphId,
      pathLength: path.length,
    });
  }

  /**
   * Apply layout algorithm
   */
  private applyLayout(params: Record<string, any>): void {
    const { graphId, layout } = params;
    if (!graphId || !layout) return;

    // Layout is handled by ReactFlow, just update the config
    this.context.componentState.set(`${graphId}.layout`, layout);
    logger.debug("Graph layout applied", {
      component: "GraphExecutor",
      graphId,
      layoutType: layout.type,
    });
  }

  /**
   * Clear graph
   */
  private clear(params: Record<string, any>): void {
    const { graphId } = params;
    if (!graphId) return;

    this.context.componentState.set(`${graphId}.nodes`, []);
    this.context.componentState.set(`${graphId}.edges`, []);
    logger.debug("Graph cleared", {
      component: "GraphExecutor",
      graphId,
    });
  }
}

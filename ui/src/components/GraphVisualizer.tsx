import React, { useMemo, useState, useRef, useCallback } from "react";
import { Plus, Minus, RotateCcw } from "lucide-react";

interface Node {
  id: string;
  label: string;
}

interface Edge {
  from: string;
  to: string;
}

interface GraphVisualizerProps {
  dotString: string;
  className?: string;
}

export function GraphVisualizer({
  dotString,
  className,
}: GraphVisualizerProps) {
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const svgRef = useRef<SVGSVGElement>(null);

  const { nodes, edges } = useMemo(() => {
    const nodes: Node[] = [];
    const edges: Edge[] = [];

    // Parse nodes
    const nodeRegex = /(\d+)\s*\[\s*label\s*=\s*"([^"]+)"\s*\]/g;
    let match;

    while ((match = nodeRegex.exec(dotString)) !== null) {
      nodes.push({
        id: match[1],
        label: match[2],
      });
    }

    // Parse edges
    const edgeRegex = /(\d+)\s*->\s*(\d+)/g;
    while ((match = edgeRegex.exec(dotString)) !== null) {
      edges.push({
        from: match[1],
        to: match[2],
      });
    }

    return { nodes, edges };
  }, [dotString]);

  const layout = useMemo(() => {
    const nodeMap = new Map<string, { x: number; y: number; level: number }>();
    const inDegree = new Map<string, number>();
    const adjList = new Map<string, string[]>();

    // Initialize
    nodes.forEach((node) => {
      inDegree.set(node.id, 0);
      adjList.set(node.id, []);
    });

    // Build adjacency list and calculate in-degrees
    edges.forEach((edge) => {
      adjList.get(edge.from)?.push(edge.to);
      inDegree.set(edge.to, (inDegree.get(edge.to) || 0) + 1);
    });

    // Topological sort to determine levels
    const queue: string[] = [];
    const levels = new Map<string, number>();

    // Find nodes with no incoming edges
    inDegree.forEach((degree, nodeId) => {
      if (degree === 0) {
        queue.push(nodeId);
        levels.set(nodeId, 0);
      }
    });

    let maxLevel = 0;
    while (queue.length > 0) {
      const current = queue.shift()!;
      const currentLevel = levels.get(current) || 0;

      adjList.get(current)?.forEach((neighbor) => {
        const newInDegree = (inDegree.get(neighbor) || 0) - 1;
        inDegree.set(neighbor, newInDegree);

        if (newInDegree === 0) {
          const newLevel = currentLevel + 1;
          levels.set(neighbor, newLevel);
          maxLevel = Math.max(maxLevel, newLevel);
          queue.push(neighbor);
        }
      });
    }

    // Group nodes by level
    const levelGroups = new Map<number, string[]>();
    levels.forEach((level, nodeId) => {
      if (!levelGroups.has(level)) {
        levelGroups.set(level, []);
      }
      levelGroups.get(level)?.push(nodeId);
    });

    // Position nodes
   const levelHeight = 80;
    const nodeSpacing = 180;

    levelGroups.forEach((nodesInLevel, level) => {
      const startX = Math.max(0, (600 - nodesInLevel.length * nodeSpacing) / 2);

      nodesInLevel.forEach((nodeId, index) => {
        nodeMap.set(nodeId, {
          x: startX + index * nodeSpacing,
          y: level * levelHeight + 30,
          level,
        });
      });
    });

    return { nodeMap, maxLevel };
  }, [nodes, edges]);

  const svgHeight = Math.max(300, (layout.maxLevel + 1) * 80 + 60);
  const svgWidth = 800;

  // Zoom controls
  const handleZoomIn = useCallback(() => {
    setZoom((prev) => Math.min(prev * 1.2, 3));
  }, []);

  const handleZoomOut = useCallback(() => {
    setZoom((prev) => Math.max(prev / 1.2, 0.3));
  }, []);

  const handleReset = useCallback(() => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
  }, []);

  // Mouse events for panning
  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      setIsDragging(true);
      setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
    },
    [pan]
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!isDragging) return;
      setPan({
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y,
      });
    },
    [isDragging, dragStart]
  );

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  // Wheel zoom
  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setZoom((prev) => Math.max(0.3, Math.min(3, prev * delta)));
  }, []);

  return (
    <div className={`w-full h-full overflow-hidden ${className}`}>
      <div className="pl-0 pr-4 pt-4 pb-4">
        <div className="flex items-center justify-between mb-3">
          <div>
            <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Computational Flow
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              Visual representation of the computation being verified
            </p>
          </div>
          <div className="flex items-center space-x-1">
            <button
              onClick={handleZoomOut}
              className="p-1 rounded bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
              title="Zoom out"
            >
              <Minus className="h-3 w-3 text-gray-600 dark:text-gray-400" />
            </button>
            <button
              onClick={handleReset}
              className="p-1 rounded bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
              title="Reset zoom"
            >
              <RotateCcw className="h-3 w-3 text-gray-600 dark:text-gray-400" />
            </button>
            <button
              onClick={handleZoomIn}
              className="p-1 rounded bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
              title="Zoom in"
            >
              <Plus className="h-3 w-3 text-gray-600 dark:text-gray-400" />
            </button>
          </div>
        </div>
        <div className="bg-white dark:bg-gray-950 rounded-lg border border-gray-200 dark:border-gray-700 p-2 h-80 overflow-hidden relative">
          <svg
            ref={svgRef}
            width="100%"
            height="100%"
            viewBox={`0 0 ${svgWidth} ${svgHeight}`}
            className="font-mono cursor-move"
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
            onMouseLeave={handleMouseUp}
            onWheel={handleWheel}
            style={{ userSelect: "none" }}
          >
            <g transform={`translate(${pan.x}, ${pan.y}) scale(${zoom})`}>
              {/* Render edges */}
              {edges.map((edge, index) => {
                const fromPos = layout.nodeMap.get(edge.from);
                const toPos = layout.nodeMap.get(edge.to);

                if (!fromPos || !toPos) return null;

                const x1 = fromPos.x + 80; // Center of from node
                const y1 = fromPos.y + 20; // Bottom of from node
                const x2 = toPos.x + 80; // Center of to node
                const y2 = toPos.y; // Top of to node

                return (
                  <g key={`edge-${index}`}>
                    <line
                      x1={x1}
                      y1={y1}
                      x2={x2}
                      y2={y2}
                      stroke="currentColor"
                      strokeWidth="1.5"
                      className="text-gray-400 dark:text-gray-500"
                      markerEnd="url(#arrowhead)"
                    />
                  </g>
                );
              })}

              {/* Render nodes */}
              {nodes.map((node) => {
                const pos = layout.nodeMap.get(node.id);
                if (!pos) return null;

                const isOperation =
                  node.label.includes("Mul") || node.label.includes("Add");
                const isLoad = node.label.includes("Load");
                const isCopy = node.label.includes("Copy");

                // Truncation that preserves tensor shapes
                const getDisplayText = (label: string) => {
                  if (label.length <= 24) return label;
                  
                  // If there's a tensor shape (brackets), try to preserve it
                  const lastBracket = label.lastIndexOf('[');
                  const closingBracket = label.lastIndexOf(']');
                  
                  if (lastBracket > 10 && closingBracket > lastBracket && label.length > 24) {
                    // Truncate before the tensor shape
                    const beforeShape = label.substring(0, lastBracket - 1);
                    const shape = label.substring(lastBracket);
                    if (beforeShape.length > 15) {
                      return beforeShape.substring(0, 12) + "..." + shape;
                    }
                    return beforeShape + shape;
                  }
                  
                  // Default truncation
                  return label.substring(0, 21) + "...";
                };

                return (
                  <g key={node.id}>
                    <rect
                      x={pos.x}
                      y={pos.y}
                      width="160"
                      height="40"
                      rx="6"
                      fill={
                        isOperation
                          ? "#dbeafe" // blue-100
                          : isLoad
                          ? "#dcfce7" // green-100
                          : isCopy
                          ? "#fef3c7" // yellow-100
                          : "#f3f4f6" // gray-100
                      }
                      stroke={
                        isOperation
                          ? "#93c5fd" // blue-300
                          : isLoad
                          ? "#86efac" // green-300
                          : isCopy
                          ? "#fcd34d" // yellow-300
                          : "#d1d5db" // gray-300
                      }
                      strokeWidth="1.5"
                    />
                    <text
                      x={pos.x + 80}
                      y={pos.y + 25}
                      textAnchor="middle"
                      fontSize="10"
                      fill="currentColor"
                      className="text-gray-700 dark:text-gray-700"
                      style={{ pointerEvents: "none" }}
                    >
                      {getDisplayText(node.label)}
                    </text>
                  </g>
                );
              })}

              {/* Arrow marker definition */}
              <defs>
                <marker
                  id="arrowhead"
                  markerWidth="10"
                  markerHeight="7"
                  refX="9"
                  refY="3.5"
                  orient="auto"
                >
                  <polygon
                    points="0 0, 10 3.5, 0 7"
                    fill="currentColor"
                    className="text-gray-400 dark:text-gray-500"
                  />
                </marker>
              </defs>
            </g>
          </svg>
        </div>
      </div>
    </div>
  );
}

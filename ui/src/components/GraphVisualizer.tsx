import React, { useMemo } from 'react';

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

export function GraphVisualizer({ dotString, className }: GraphVisualizerProps) {
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

  // Simple layout algorithm - arrange nodes in layers
  const layout = useMemo(() => {
    const nodeMap = new Map<string, { x: number; y: number; level: number }>();
    const inDegree = new Map<string, number>();
    const adjList = new Map<string, string[]>();

    // Initialize
    nodes.forEach(node => {
      inDegree.set(node.id, 0);
      adjList.set(node.id, []);
    });

    // Build adjacency list and calculate in-degrees
    edges.forEach(edge => {
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

      adjList.get(current)?.forEach(neighbor => {
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
    const nodeWidth = 120;
    const nodeHeight = 40;
    const levelHeight = 80;
    const nodeSpacing = 140;

    levelGroups.forEach((nodesInLevel, level) => {
      const startX = Math.max(0, (600 - (nodesInLevel.length * nodeSpacing)) / 2);
      
      nodesInLevel.forEach((nodeId, index) => {
        nodeMap.set(nodeId, {
          x: startX + (index * nodeSpacing),
          y: level * levelHeight + 30,
          level,
        });
      });
    });

    return { nodeMap, maxLevel };
  }, [nodes, edges]);

  const svgHeight = Math.max(300, (layout.maxLevel + 1) * 80 + 60);

  return (
    <div className={`w-full h-full overflow-auto ${className}`}>
      <div className="p-4">
        <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
          Computation Graph
        </h3>
        <div className="bg-white dark:bg-gray-950 rounded-lg border border-gray-200 dark:border-gray-700 p-4">
          <svg
            width="100%"
            height={svgHeight}
            viewBox={`0 0 600 ${svgHeight}`}
            className="font-mono"
          >
            {/* Render edges */}
            {edges.map((edge, index) => {
              const fromPos = layout.nodeMap.get(edge.from);
              const toPos = layout.nodeMap.get(edge.to);
              
              if (!fromPos || !toPos) return null;

              const x1 = fromPos.x + 60; // Center of from node
              const y1 = fromPos.y + 20; // Bottom of from node
              const x2 = toPos.x + 60; // Center of to node  
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

              const isOperation = node.label.includes('Mul') || node.label.includes('Add');
              const isLoad = node.label.includes('Load');
              const isCopy = node.label.includes('Copy');

              return (
                <g key={node.id}>
                  <rect
                    x={pos.x}
                    y={pos.y}
                    width="120"
                    height="40"
                    rx="6"
                    fill={
                      isOperation
                        ? '#dbeafe' // blue-100
                        : isLoad
                        ? '#dcfce7' // green-100
                        : isCopy
                        ? '#fef3c7' // yellow-100
                        : '#f3f4f6' // gray-100
                    }
                    stroke={
                      isOperation
                        ? '#93c5fd' // blue-300
                        : isLoad
                        ? '#86efac' // green-300
                        : isCopy
                        ? '#fcd34d' // yellow-300
                        : '#d1d5db' // gray-300
                    }
                    strokeWidth="1.5"
                  />
                  <text
                    x={pos.x + 60}
                    y={pos.y + 25}
                    textAnchor="middle"
                    fontSize="10"
                    fill="currentColor"
                    className="text-gray-700 dark:text-gray-300"
                  >
                    {node.label.length > 16 ? `${node.label.substring(0, 13)}...` : node.label}
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
          </svg>
        </div>
      </div>
    </div>
  );
} 
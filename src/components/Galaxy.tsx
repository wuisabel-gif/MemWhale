import * as d3 from "d3";
import { Sparkles } from "lucide-react";
import { useEffect, useRef } from "react";
import type { GraphLink, GraphNode, GraphPayload } from "../types";

type SimNode = GraphNode & d3.SimulationNodeDatum;
type SimLink = GraphLink & d3.SimulationLinkDatum<SimNode>;

export function Galaxy({
  graph,
  selectedId,
  onSelect
}: {
  graph: GraphPayload;
  selectedId: string | null;
  onSelect: (id: string) => void;
}) {
  const svgRef = useRef<SVGSVGElement | null>(null);

  useEffect(() => {
    const svgEl = svgRef.current;
    if (!svgEl) return;

    const width = svgEl.clientWidth || 720;
    const height = svgEl.clientHeight || 560;
    const nodes: SimNode[] = graph.nodes.map((node) => ({ ...node }));
    const links: SimLink[] = graph.links.map((link) => ({ ...link }));
    const svg = d3.select(svgEl);
    svg.selectAll("*").remove();

    const defs = svg.append("defs");
    const glow = defs.append("filter").attr("id", "node-glow");
    glow.append("feGaussianBlur").attr("stdDeviation", "3.2").attr("result", "coloredBlur");
    const merge = glow.append("feMerge");
    merge.append("feMergeNode").attr("in", "coloredBlur");
    merge.append("feMergeNode").attr("in", "SourceGraphic");

    const root = svg.append("g");
    svg.call(
      d3
        .zoom<SVGSVGElement, unknown>()
        .scaleExtent([0.35, 4])
        .on("zoom", (event) => root.attr("transform", event.transform))
    );

    root
      .append("g")
      .attr("class", "starfield")
      .selectAll("circle")
      .data(d3.range(100))
      .join("circle")
      .attr("cx", () => Math.random() * width)
      .attr("cy", () => Math.random() * height)
      .attr("r", () => Math.random() * 1.25 + 0.25);

    const link = root
      .append("g")
      .attr("class", "links")
      .selectAll<SVGLineElement, SimLink>("line")
      .data(links)
      .join("line")
      .attr("stroke-width", (d) => Math.max(1, Math.min(5, d.weight)));

    const node = root
      .append("g")
      .attr("class", "nodes")
      .selectAll<SVGGElement, SimNode>("g")
      .data(nodes)
      .join("g")
      .attr("class", (d) => `graph-node ${d.node_type} ${d.id === selectedId ? "selected" : ""}`)
      .on("click", (_, d) => onSelect(d.id))
      .on("dblclick", (_, d) => {
        onSelect(d.id);
        svg
          .transition()
          .duration(450)
          .call(
            d3.zoom<SVGSVGElement, unknown>().transform,
            d3.zoomIdentity.translate(width / 2 - (d.x ?? 0) * 1.8, height / 2 - (d.y ?? 0) * 1.8).scale(1.8)
          );
      });

    node
      .append("circle")
      .attr("r", (d) => nodeRadius(d))
      .attr("filter", "url(#node-glow)");

    node
      .append("text")
      .text((d) => d.label)
      .attr("x", (d) => nodeRadius(d) + 8)
      .attr("y", 4);

    const simulation = d3
      .forceSimulation(nodes)
      .force(
        "link",
        d3
          .forceLink<SimNode, SimLink>(links)
          .id((d) => d.id)
          .distance((d) => (d.relation === "co_occurs" ? 82 : 135))
          .strength(0.58)
      )
      .force("charge", d3.forceManyBody().strength(-360))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .force("collision", d3.forceCollide<SimNode>().radius((d) => nodeRadius(d) + 18))
      .on("tick", () => {
        link
          .attr("x1", (d) => (d.source as SimNode).x ?? 0)
          .attr("y1", (d) => (d.source as SimNode).y ?? 0)
          .attr("x2", (d) => (d.target as SimNode).x ?? 0)
          .attr("y2", (d) => (d.target as SimNode).y ?? 0);
        node.attr("transform", (d) => `translate(${d.x ?? 0},${d.y ?? 0})`);
      });

    node.call(
      d3
        .drag<SVGGElement, SimNode>()
        .on("start", (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x;
          d.fy = d.y;
        })
        .on("drag", (event, d) => {
          d.fx = event.x;
          d.fy = event.y;
        })
        .on("end", (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null;
          d.fy = null;
        })
    );

    return () => {
      simulation.stop();
    };
  }, [graph, onSelect, selectedId]);

  if (graph.nodes.length === 0) {
    return (
      <div className="empty-galaxy">
        <Sparkles size={32} />
        <h2>Import a note to form the first constellation</h2>
        <p>MemoryWhale will extract concepts and connect them as local graph nodes.</p>
      </div>
    );
  }

  return <svg ref={svgRef} className="galaxy" role="img" aria-label="Interactive knowledge graph" />;
}

function nodeRadius(node: GraphNode) {
  const base = node.node_type === "document" ? 14 : node.node_type === "command" ? 12 : 9;
  return Math.max(base, Math.min(30, base + node.weight * 1.35));
}

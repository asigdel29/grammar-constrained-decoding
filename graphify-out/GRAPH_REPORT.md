# Graph Report - grammar-constrained-decoding  (2026-11-27)

## Corpus Check
- 11 files · ~2,407 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 126 nodes · 179 edges · 10 communities (3 shown, 7 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- benchmark.py
- compile_schema
- Grammar
- .allowed_ids
- json
- schema.rs
- test_conformance.py
- Grammar
- grammar-decode.rs

## God Nodes (most connected - your core abstractions)
1. `compile_schema()` - 13 edges
2. `benchmark()` - 11 edges
3. `Grammar` - 8 edges
4. `Grammar` - 7 edges
5. `Prefix` - 7 edges
6. `Conformance` - 7 edges
7. `compile_model()` - 6 edges
8. `main()` - 6 edges
9. `node()` - 6 edges
10. `main()` - 5 edges

## Surprising Connections (you probably didn't know these)
- `benchmark()` --calls--> `compile_schema()`  [EXTRACTED]
  python/grammar_decode/benchmark.py → python/grammar_decode/__init__.py
- `main()` --calls--> `compile_schema()`  [EXTRACTED]
  python/grammar_decode/__main__.py → python/grammar_decode/__init__.py

## Import Cycles
- None detected.

## Communities (10 total, 7 thin omitted)

### Community 0 - "benchmark.py"
Cohesion: 0.11
Nodes (4): benchmark(), measure(), workloads(), BenchmarkTests

### Community 1 - "compile_schema"
Cohesion: 0.15
Nodes (4): compile_model(), compile_schema(), SchemaError, Conformance

### Community 5 - "schema.rs"
Cohesion: 0.24
Nodes (7): append(), INTEGER, literal(), MAX_PATTERN_BYTES, node(), pattern(), STRING

## Knowledge Gaps
- **3 isolated node(s):** `STRING`, `INTEGER`, `MAX_PATTERN_BYTES`
  These have ≤1 connection - possible missing edges. (Counts symbols only; 57 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `compile_schema()` connect `compile_schema` to `benchmark.py`, `json`, `Grammar`?**
  _High betweenness centrality (0.156) - this node is a cross-community bridge._
- **What connects `STRING`, `INTEGER`, `MAX_PATTERN_BYTES` to the rest of the system?**
  _3 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `benchmark.py` be split into smaller, more focused modules?**
  _Cohesion score 0.11067193675889328 - nodes in this community are weakly interconnected._
- **Why does `Grammar` connect `Grammar` to `compile_schema`?**
  _High betweenness centrality (0.083) - this node is a cross-community bridge._
# Graph Report - grammar-constrained-decoding  (2026-11-26)

## Corpus Check
- 5 files · ~1,282 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 74 nodes · 115 edges · 7 communities (3 shown, 4 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- benchmark.py
- __init__.py
- test_conformance.py
- compile_schema
- __main__.py
- test_benchmark.py
- Grammar

## God Nodes (most connected - your core abstractions)
1. `compile_schema()` - 15 edges
2. `benchmark()` - 12 edges
3. `_pattern()` - 7 edges
4. `Grammar` - 7 edges
5. `compile_model()` - 7 edges
6. `Conformance` - 7 edges
7. `SchemaError` - 6 edges
8. `main()` - 6 edges
9. `_literal()` - 5 edges
10. `main()` - 5 edges

## Surprising Connections (you probably didn't know these)
- `benchmark()` --calls--> `compile_schema()`  [EXTRACTED]
  grammar_decode/benchmark.py → grammar_decode/__init__.py
- `main()` --calls--> `compile_schema()`  [EXTRACTED]
  grammar_decode/__main__.py → grammar_decode/__init__.py

## Import Cycles
- None detected.

## Communities (7 total, 4 thin omitted)

### Community 0 - "benchmark.py"
Cohesion: 0.19
Nodes (3): benchmark(), measure(), workloads()

### Community 1 - "__init__.py"
Cohesion: 0.23
Nodes (3): _literal(), _pattern(), SchemaError

### Community 3 - "compile_schema"
Cohesion: 0.25
Nodes (3): compile_model(), compile_schema(), Conformance

## Knowledge Gaps
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `compile_schema()` connect `compile_schema` to `benchmark.py`, `__init__.py`, `test_conformance.py`, `__main__.py`, `Grammar`?**
  _High betweenness centrality (0.424) - this node is a cross-community bridge._
- **Why does `benchmark()` connect `benchmark.py` to `compile_schema`, `__main__.py`, `test_benchmark.py`?**
  _High betweenness centrality (0.226) - this node is a cross-community bridge._
- **Why does `Grammar` connect `Grammar` to `__init__.py`, `compile_schema`?**
  _High betweenness centrality (0.179) - this node is a cross-community bridge._
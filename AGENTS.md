# Repository instructions

Read the public contract before changing the compiler. [System design](docs/system-design.md)

- Keep byte prefixes intact across token boundaries.
- Check the complete prefix before admitting the end token.
- Reject unsupported constraints at compile time.
- Preserve canonical field order and compact output.
- Add a conformance case for each change to schema support or token eligibility.
- Run `make check` in an environment with the Pydantic extra.
- Run the mask benchmark after changes to pattern construction or vocabulary traversal.
- Document workload, environment, and raw timings for performance comparisons.
- Update both the support table and operating examples when the public contract changes.

Use the operating guide for installation and reproduction commands. [Operating guide](docs/operations.md)

Use `graphify query "compile_schema"` to locate related code. Run `make graph` after source structure changes.

# Run conformance tests and reference examples.
PYTHON ?= python3
.PHONY: check demo bench
check:
	$(PYTHON) -m unittest discover -s tests -v
demo:
	$(PYTHON) -m grammar_decode
bench:
	$(PYTHON) -m grammar_decode.benchmark $(ARGS)

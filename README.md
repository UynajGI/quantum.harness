# Quantum Many-Body Physics Harness

Research harness for quantum many-body physics with tensor network methods. Covers theoretical foundations (second quantization, Green's functions, Fermi liquids) and computational approaches (MPS, PEPS, DMRG, TEBD, MERA, TN contractions).

## Quick Start

1. Install [Ion](https://github.com/Roger-luo/Ion): `curl -fsSL https://raw.githubusercontent.com/Roger-luo/Ion/main/install.sh | sh`
2. Run `make setup` (minimal bootstrap — installs Ion skills only)
3. Run `make help` to see optional installs. Use `make install quimb` for the Python quimb stack, or `make install quarto` for HTML rendering.
4. Ask a concrete quantum many-body problem; the harness routes to problem skills.
5. Use `knowledge-base/` only as factual reference, not as a learning path.

## Structure

- `raw/` — Raw materials (git-ignored, any format)
- `knowledge-base/` — Non-actionable reference notes for papers, definitions, equations, and citations
- `templates/` — HTML template used by the render tool
- `tools/` — CLI scripts (including `render`), MCP configs, and local problem skills
- `Makefile` — Setup and daily workflow targets
- `AGENTS.md` — AI instructions (canonical); `CLAUDE.md` is a one-liner pointer to it

.PHONY: setup test clean help install $(addprefix install-,$(INSTALLABLE))

INSTALLABLE := quarto quimb

help: ## Show available targets and installable tools
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Installable tools — run 'make install <name>':"
	@for t in $(INSTALLABLE); do echo "  $$t"; done

setup: ## Minimal bootstrap — Ion + skills only
	@command -v ion >/dev/null 2>&1 || { echo "Ion not found. Install: curl -fsSL https://raw.githubusercontent.com/Roger-luo/Ion/main/install.sh | sh"; exit 1; }
	ion add
	@echo ""
	@echo "Base setup complete. Install tools as you need them:"
	@echo "  make install <tool>   (see 'make help' for the list)"

install: ## Install a specific tool on demand. Usage: make install <tool>
	@tool="$(filter-out install,$(MAKECMDGOALS))"; \
	if [ -z "$$tool" ]; then \
	  echo "Usage: make install <tool>"; \
	  echo "Installable: $(INSTALLABLE)"; exit 1; \
	fi; \
	if ! echo " $(INSTALLABLE) " | grep -q " $$tool "; then \
	  echo "Unknown tool '$$tool'. Installable: $(INSTALLABLE)"; exit 1; \
	fi; \
	$(MAKE) install-$$tool

$(INSTALLABLE):
	@:

install-quarto: ## Install Quarto + TinyTeX for content rendering
	@command -v quarto >/dev/null && echo "Quarto already installed" || { echo "Install Quarto: https://quarto.org/docs/get-started/"; exit 1; }
	@quarto install tinytex --no-prompt 2>/dev/null || true

install-quimb: ## Install quimb + common numerical dependencies into .venv
	@command -v uv >/dev/null 2>&1 || { echo "uv not found. Install uv first: https://docs.astral.sh/uv/getting-started/installation/"; exit 1; }
	@uv venv .venv
	@uv pip install quimb cotengra autoray opt_einsum numpy scipy matplotlib jupyter ipykernel
	@echo "quimb environment ready in .venv"

render: ## Render a markdown file to HTML. Usage: make render FILE=<path.md>
	@if [ -z "$(FILE)" ]; then echo "Usage: make render FILE=<path.md>"; exit 1; fi
	tools/cli/render "$(FILE)"

clean: ## Remove generated HTML artifacts
	find . -name '*.html' -not -path './templates/*' -delete

#!/bin/bash
# Atomic shell helper for /setup-julia. Idempotent.
#
# Usage:
#   tools/cli/setup-julia.sh install [--version X.Y.Z]
#   tools/cli/setup-julia.sh mirror <url>             # write JULIA_PKG_SERVER into startup.jl
#   tools/cli/setup-julia.sh mirror clear             # remove the mirror line
#   tools/cli/setup-julia.sh instantiate <project_dir>
#   tools/cli/setup-julia.sh verify <project_dir> <package_name>
#
# All commands are idempotent; safe to re-run. Designed to be invoked locally
# OR via ssh against a remote cluster (the script is self-contained and POSIX).

set -euo pipefail

cmd="${1:-help}"

case "$cmd" in
  install)
    shift
    version="release"
    while [ $# -gt 0 ]; do
      case "$1" in
        --version) version="$2"; shift 2;;
        *) shift;;
      esac
    done
    if command -v julia >/dev/null 2>&1; then
      echo "Julia already on PATH: $(julia --version)"
      exit 0
    fi
    if ! command -v juliaup >/dev/null 2>&1; then
      echo "Installing juliaup..."
      curl -fsSL https://install.julialang.org -o /tmp/juliaup-install.sh
      sh /tmp/juliaup-install.sh --yes --default-channel "$version"
      rm -f /tmp/juliaup-install.sh
      # juliaup puts itself in ~/.juliaup/bin; user needs to source profile or open new shell
      export PATH="$HOME/.juliaup/bin:$PATH"
    else
      juliaup add "$version"
      juliaup default "$version"
    fi
    echo "Julia installed: $(~/.juliaup/bin/julia --version 2>/dev/null || julia --version)"
    ;;

  mirror)
    shift
    url="${1:-}"
    config_dir="$HOME/.julia/config"
    startup="$config_dir/startup.jl"
    mkdir -p "$config_dir"
    touch "$startup"
    if [ "$url" = "clear" ]; then
      grep -v 'JULIA_PKG_SERVER' "$startup" > "$startup.tmp" && mv "$startup.tmp" "$startup"
      echo "Cleared JULIA_PKG_SERVER from $startup"
    elif [ -n "$url" ]; then
      grep -v 'JULIA_PKG_SERVER' "$startup" > "$startup.tmp" || true
      echo "ENV[\"JULIA_PKG_SERVER\"] = \"$url\"" >> "$startup.tmp"
      mv "$startup.tmp" "$startup"
      echo "Set JULIA_PKG_SERVER = $url in $startup"
    else
      echo "Usage: $0 mirror <url|clear>" >&2
      exit 2
    fi
    ;;

  instantiate)
    shift
    project_dir="${1:-julia-env}"
    if [ ! -d "$project_dir" ]; then
      echo "Project dir not found: $project_dir" >&2
      exit 1
    fi
    if [ ! -f "$project_dir/Project.toml" ]; then
      echo "No Project.toml in $project_dir — nothing to instantiate" >&2
      exit 1
    fi
    julia_bin="$(command -v julia 2>/dev/null || echo "$HOME/.juliaup/bin/julia")"
    if [ ! -x "$julia_bin" ]; then
      echo "Julia not found — run '$0 install' first" >&2
      exit 1
    fi
    echo "Instantiating $project_dir using $julia_bin..."
    "$julia_bin" --project="$project_dir" -e 'using Pkg; Pkg.instantiate(); Pkg.precompile()'
    ;;

  verify)
    shift
    project_dir="${1:-julia-env}"
    pkg="${2:-}"
    if [ -z "$pkg" ]; then
      echo "Usage: $0 verify <project_dir> <package_name>" >&2
      exit 2
    fi
    julia_bin="$(command -v julia 2>/dev/null || echo "$HOME/.juliaup/bin/julia")"
    "$julia_bin" --project="$project_dir" -e "using $pkg; println(\"ok: $pkg loaded\")"
    ;;

  help|*)
    cat <<EOF
Usage:
  $0 install [--version X.Y.Z]
  $0 mirror <url|clear>
  $0 instantiate [project_dir]   # default julia-env
  $0 verify <project_dir> <package_name>
EOF
    ;;
esac

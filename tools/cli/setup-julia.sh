#!/bin/bash
# Atomic shell helper for /setup-julia. Follows the Jinguo-group recipe:
#   https://book.jinguo-group.science/stable/chap2/julia-setup/
#
# Step 1 of the guide: install Julia via juliaup (curl install.julialang.org).
# Step 2 of the guide: configure the package mirror (JULIA_PKG_SERVER) and the
# juliaup mirror (JULIAUP_SERVER). Both must be set BEFORE step 1 if the user
# is in mainland China — otherwise juliaup downloads from AWS S3 (slow).
#
# Mirror used (per guide): Nanjing University.
#   JULIAUP_SERVER = https://mirror.nju.edu.cn/julia-releases/
#   JULIA_PKG_SERVER = https://mirrors.nju.edu.cn/julia
#
# All commands are idempotent; safe to re-run. Designed to be invoked locally
# OR via ssh against a remote cluster (the script is self-contained POSIX).
#
# Usage:
#   tools/cli/setup-julia.sh mirror {--region <name> | --url <url> | clear}
#   tools/cli/setup-julia.sh install [--region <name>] [--version X.Y.Z]
#   tools/cli/setup-julia.sh revise-startup
#   tools/cli/setup-julia.sh instantiate [project_dir]
#   tools/cli/setup-julia.sh verify <project_dir> <package_name>

set -euo pipefail

# ---------------- region → mirror map ----------------
region_to_pkg_mirror() {
  case "${1:-}" in
    mainland_china) echo "https://mirrors.nju.edu.cn/julia";;
    *)              echo "";;
  esac
}
region_to_juliaup_mirror() {
  case "${1:-}" in
    mainland_china) echo "https://mirror.nju.edu.cn/julia-releases/";;
    *)              echo "";;
  esac
}

# ---------------- shell rc detection ----------------
detect_rc() {
  case "${SHELL:-}" in
    */zsh)  echo "$HOME/.zshrc";;
    */bash) echo "$HOME/.bashrc";;
    *)      echo "$HOME/.profile";;
  esac
}

# Idempotent line-replace in shell rc: keep one canonical export <var>=<value>.
upsert_rc_export() {
  local rc="$1" var="$2" val="$3"
  touch "$rc"
  grep -v "^export $var=" "$rc" > "$rc.tmp" || true
  echo "export $var=\"$val\"" >> "$rc.tmp"
  mv "$rc.tmp" "$rc"
}
clear_rc_export() {
  local rc="$1" var="$2"
  [ -f "$rc" ] || return 0
  grep -v "^export $var=" "$rc" > "$rc.tmp" || true
  mv "$rc.tmp" "$rc"
}

# Idempotent JULIA_PKG_SERVER write in startup.jl.
upsert_startup_pkg_server() {
  local url="$1"
  local config_dir="$HOME/.julia/config"
  local startup="$config_dir/startup.jl"
  mkdir -p "$config_dir"
  touch "$startup"
  grep -v 'JULIA_PKG_SERVER' "$startup" > "$startup.tmp" || true
  echo "ENV[\"JULIA_PKG_SERVER\"] = \"$url\"" >> "$startup.tmp"
  mv "$startup.tmp" "$startup"
}

# Revise-startup snippet (per guide).
write_revise_startup() {
  local config_dir="$HOME/.julia/config"
  local startup="$config_dir/startup.jl"
  mkdir -p "$config_dir"
  touch "$startup"
  if grep -q "using Revise" "$startup"; then
    return 0
  fi
  cat >> "$startup" <<'EOF'
try
    using Revise
catch e
    @warn "fail to load Revise."
end
EOF
}

# ---------------- subcommands ----------------

cmd="${1:-help}"
shift || true

case "$cmd" in

  mirror)
    # Set both JULIAUP_SERVER (shell rc) and JULIA_PKG_SERVER (shell rc + startup.jl).
    region=""
    url=""
    while [ $# -gt 0 ]; do
      case "$1" in
        --region) region="$2"; shift 2;;
        --url)    url="$2"; shift 2;;
        clear)    region="clear"; shift;;
        *)        url="$1"; shift;;
      esac
    done

    rc=$(detect_rc)
    if [ "$region" = "clear" ] || [ "$url" = "clear" ]; then
      clear_rc_export "$rc" JULIAUP_SERVER
      clear_rc_export "$rc" JULIA_PKG_SERVER
      [ -f "$HOME/.julia/config/startup.jl" ] && \
        sed -i.bak '/JULIA_PKG_SERVER/d' "$HOME/.julia/config/startup.jl" && \
        rm -f "$HOME/.julia/config/startup.jl.bak"
      echo "Mirror cleared from $rc and ~/.julia/config/startup.jl"
      exit 0
    fi

    pkg_url="$url"
    juliaup_url=""
    if [ -n "$region" ]; then
      pkg_url=$(region_to_pkg_mirror "$region")
      juliaup_url=$(region_to_juliaup_mirror "$region")
    fi
    if [ -z "$pkg_url" ]; then
      echo "Usage: $0 mirror {--region mainland_china | --url <url> | clear}" >&2
      exit 2
    fi

    upsert_rc_export "$rc" JULIA_PKG_SERVER "$pkg_url"
    upsert_startup_pkg_server "$pkg_url"
    if [ -n "$juliaup_url" ]; then
      upsert_rc_export "$rc" JULIAUP_SERVER "$juliaup_url"
    fi
    echo "Mirror configured:"
    echo "  JULIA_PKG_SERVER = $pkg_url   (in $rc and ~/.julia/config/startup.jl)"
    [ -n "$juliaup_url" ] && echo "  JULIAUP_SERVER   = $juliaup_url   (in $rc)"
    echo "Open a new shell or 'source $rc' for the env vars to take effect."
    ;;

  install)
    # Step 1 (per guide): install Julia via juliaup. Uses JULIAUP_SERVER if set.
    region=""
    version="release"
    while [ $# -gt 0 ]; do
      case "$1" in
        --region)  region="$2"; shift 2;;
        --version) version="$2"; shift 2;;
        *) shift;;
      esac
    done

    # Apply mirror first (Step 2 before Step 1 if region is given), so the curl
    # juliaup-installer respects JULIAUP_SERVER.
    if [ -n "$region" ]; then
      "$0" mirror --region "$region"
      # Sourcing the rc isn't enough mid-script; export inline for this session.
      juliaup_url=$(region_to_juliaup_mirror "$region")
      pkg_url=$(region_to_pkg_mirror "$region")
      [ -n "$juliaup_url" ] && export JULIAUP_SERVER="$juliaup_url"
      [ -n "$pkg_url" ]     && export JULIA_PKG_SERVER="$pkg_url"
    fi

    if command -v julia >/dev/null 2>&1; then
      echo "Julia already on PATH: $(julia --version)"
    else
      if ! command -v juliaup >/dev/null 2>&1; then
        echo "Installing juliaup (with JULIAUP_SERVER=${JULIAUP_SERVER:-default})..."
        curl -fsSL https://install.julialang.org -o /tmp/juliaup-install.sh
        sh /tmp/juliaup-install.sh --yes --default-channel "$version"
        rm -f /tmp/juliaup-install.sh
        export PATH="$HOME/.juliaup/bin:$PATH"
      else
        juliaup add "$version"
        juliaup default "$version"
      fi
      echo "Julia installed: $(julia --version 2>/dev/null || ~/.juliaup/bin/julia --version)"
    fi

    # Revise startup (per guide).
    write_revise_startup
    julia_bin="$(command -v julia 2>/dev/null || echo "$HOME/.juliaup/bin/julia")"
    if [ -x "$julia_bin" ]; then
      echo "Adding Revise to default Julia env..."
      "$julia_bin" -e 'using Pkg; "Revise" in [d.name for d in values(Pkg.dependencies())] || Pkg.add("Revise")' || true
    fi
    ;;

  revise-startup)
    write_revise_startup
    julia_bin="$(command -v julia 2>/dev/null || echo "$HOME/.juliaup/bin/julia")"
    [ -x "$julia_bin" ] && "$julia_bin" -e 'using Pkg; "Revise" in [d.name for d in values(Pkg.dependencies())] || Pkg.add("Revise")' || true
    echo "Revise startup written to ~/.julia/config/startup.jl"
    ;;

  instantiate)
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
      echo "Julia not found — run '$0 install --region <region>' first" >&2
      exit 1
    fi
    echo "Resolving + instantiating $project_dir using $julia_bin..."
    # Manifest.toml is gitignored (platform/version-specific). Pkg.resolve()
    # generates a fresh Manifest from Project.toml against this Julia's
    # registry, then Pkg.instantiate() materializes it.
    "$julia_bin" --project="$project_dir" -e 'using Pkg; Pkg.resolve(); Pkg.instantiate(); Pkg.precompile()'
    ;;

  verify)
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
$0 — Julia setup helper (Jinguo-group recipe).

Per https://book.jinguo-group.science/stable/chap2/julia-setup/

Usage:
  $0 mirror {--region mainland_china | --url <url> | clear}
      Configure JULIAUP_SERVER + JULIA_PKG_SERVER (shell rc + startup.jl).
      Use this BEFORE 'install' so the curl installer downloads through the mirror.

  $0 install [--region mainland_china] [--version X.Y.Z]
      Step 1 of the guide. Installs juliaup via curl install.julialang.org,
      respecting JULIAUP_SERVER if set. Adds Revise to startup.jl.
      With --region: applies 'mirror' first.

  $0 revise-startup
      Write the guide's Revise try/catch into ~/.julia/config/startup.jl
      and Pkg.add("Revise") in the default env.

  $0 instantiate [project_dir]
      Run Pkg.instantiate() + Pkg.precompile() in the given project (default julia-env).

  $0 verify <project_dir> <package_name>
      Smoke: 'using <package_name>' in the project. Exit 0 on success.
EOF
    ;;
esac

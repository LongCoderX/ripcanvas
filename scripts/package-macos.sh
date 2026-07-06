#!/usr/bin/env bash
set -euo pipefail

arch=""
version=""

usage() {
    cat <<'USAGE'
Usage: scripts/package-macos.sh [--arch x64|arm64|universal] [--version VERSION]

Creates a portable macOS zip package under dist/.

Examples:
  scripts/package-macos.sh
  scripts/package-macos.sh --arch x64
  scripts/package-macos.sh --arch universal --version v0.1.24
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --arch)
            arch="${2:-}"
            shift 2
            ;;
        --version)
            version="${2:-}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

if [[ -z "$arch" ]]; then
    case "$(uname -m)" in
        x86_64) arch="x64" ;;
        arm64|aarch64) arch="arm64" ;;
        *) arch="arm64" ;;
    esac
fi

case "$arch" in
    x64|arm64|universal) ;;
    *)
        echo "Unsupported arch: $arch" >&2
        echo "Expected one of: x64, arm64, universal" >&2
        exit 1
        ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dist_root="$repo_root/dist"
version_suffix=""
if [[ -n "$version" ]]; then
    version_suffix="-$version"
fi

package_name="ripcanvas-macos-$arch$version_suffix"
package_dir="$dist_root/$package_name"
zip_path="$dist_root/$package_name.zip"

build_target() {
    local target_triple="$1"
    rustup target add "$target_triple"
    cargo build --release --bin rocv --target "$target_triple"
}

copy_binary() {
    local source_path="$1"
    cp "$source_path" "$package_dir/rocv"
    chmod +x "$package_dir/rocv"
}

cd "$repo_root"
mkdir -p "$dist_root"
rm -rf "$package_dir" "$zip_path"
mkdir -p "$package_dir"

case "$arch" in
    x64)
        target_triple="x86_64-apple-darwin"
        build_target "$target_triple"
        copy_binary "$repo_root/target/$target_triple/release/rocv"
        ;;
    arm64)
        target_triple="aarch64-apple-darwin"
        build_target "$target_triple"
        copy_binary "$repo_root/target/$target_triple/release/rocv"
        ;;
    universal)
        x64_target="x86_64-apple-darwin"
        arm64_target="aarch64-apple-darwin"
        build_target "$x64_target"
        build_target "$arm64_target"
        lipo -create \
            "$repo_root/target/$x64_target/release/rocv" \
            "$repo_root/target/$arm64_target/release/rocv" \
            -output "$package_dir/rocv"
        chmod +x "$package_dir/rocv"
        ;;
esac

if [[ -f "$repo_root/assets/icon.png" ]]; then
    cp "$repo_root/assets/icon.png" "$package_dir/icon.png"
fi

cat > "$package_dir/install.sh" <<'INSTALL'
#!/usr/bin/env bash
set -euo pipefail

install_dir="${1:-$HOME/.local/bin}"
mkdir -p "$install_dir"
cp "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/rocv" "$install_dir/rocv"
chmod +x "$install_dir/rocv"

echo "Installed rocv to $install_dir/rocv"
echo "Make sure $install_dir is in your PATH."
INSTALL
chmod +x "$package_dir/install.sh"

cat > "$package_dir/uninstall.sh" <<'UNINSTALL'
#!/usr/bin/env bash
set -euo pipefail

install_dir="${1:-$HOME/.local/bin}"
rm -f "$install_dir/rocv"
echo "Removed $install_dir/rocv"
UNINSTALL
chmod +x "$package_dir/uninstall.sh"

cat > "$package_dir/README.md" <<README
# RipCanvas macOS Portable Package

Run from Terminal:

\`\`\`bash
./install.sh
rocv path/to/file.canvas
\`\`\`

The installer copies \`rocv\` to \`~/.local/bin\` by default.

To install to another directory:

\`\`\`bash
./install.sh /usr/local/bin
\`\`\`

Package architecture: $arch
README

(
    cd "$dist_root"
    zip -qr "$zip_path" "$package_name"
)

echo "Portable package created:"
echo "  $package_dir"
echo "  $zip_path"

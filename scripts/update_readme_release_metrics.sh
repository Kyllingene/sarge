#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readme="$repo_root/README.md"

tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

du_kib() {
  du -sk "$1" | awk '{print $1}'
}

format_mib_1dp() {
  awk -v kib="$1" 'BEGIN{printf "%.1fMiB", kib/1024}'
}

git clone --depth 1 "file://$repo_root" "$tmp/sarge" -q
sarge_kib="$(du_kib "$tmp/sarge")"
sarge_size="${sarge_kib}KiB"

clap_size=""
if [[ "${SARGE_SKIP_CLAP_SIZE:-}" != "1" ]]; then
  clap_repo="${SARGE_CLAP_REPO:-https://github.com/clap-rs/clap.git}"
  if git clone --depth 1 "$clap_repo" "$tmp/clap" -q; then
    clap_kib="$(du_kib "$tmp/clap")"
    clap_size="$(format_mib_1dp "$clap_kib")"
  fi
fi

python3 - "$readme" "$sarge_size" "$clap_size" <<'PY'
import re
import sys
from pathlib import Path

readme_path, sarge_size, clap_size = sys.argv[1:4]
path = Path(readme_path)
text = path.read_text(encoding="utf-8")

pattern = r"(Leads to small size:\s*`)([^`]+)(`\s*compared to clap's\s*`)([^`]+)(`\\\*)"
m = re.search(pattern, text)
if not m:
    raise SystemExit(f"README size line not found in: {readme_path}")

new_clap_size = clap_size or m.group(4)
replacement = m.group(1) + sarge_size + m.group(3) + new_clap_size + m.group(5)
text2 = re.sub(pattern, replacement, text, count=1)

if text2 == text:
    raise SystemExit("README size replacement made no changes")

path.write_text(text2, encoding="utf-8")
PY

echo "Updated README metrics: sarge=${sarge_size}, clap=${clap_size:-<unchanged>}"


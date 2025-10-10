#!/usr/bin/env bash
set -euo pipefail

version="${1:-}"
if [[ -z "$version" ]]; then
  version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
fi

if [[ -z "$version" ]]; then
  echo "could not read version from Cargo.toml" >&2
  exit 1
fi

tag="v${version#v}"

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "working tree has uncommitted changes; commit before tagging" >&2
  exit 1
fi

current_branch="$(git branch --show-current)"
if [[ "$current_branch" != "main" ]]; then
  echo "release tags should be created from main (current: $current_branch)" >&2
  exit 1
fi

if git rev-parse "$tag" >/dev/null 2>&1; then
  echo "tag already exists locally: $tag" >&2
  exit 1
fi

git fetch origin main --tags
git tag -a "$tag" -m "TailScout $tag"
git push origin main
git push origin "$tag"

echo "Pushed $tag. GitHub Actions will build and publish the release."

#!/usr/bin/env -S just --justfile

image_base := "ghcr.io/verseghy"
image_tag := "v7"
image_rust_version := "1.83"

_default:
  @just --list --unsorted

install-tools:
  cargo install cargo-deny --locked
  cargo install cargo-llvm-cov --locked

test-coverage:
  @cargo llvm-cov --ignore-filename-regex "(migration|entity|cmds)/.*" nextest

fmt:
  cargo fmt --all

clippy:
  cargo clippy --all --all-targets --all-features

[private]
build-image NAME FILE:
  podman build \
    --file "{{FILE}}" \
    --tag {{image_base}}/{{NAME}}:{{image_tag}} \
    --build-arg RUST_VERSION="{{image_rust_version}}" \
    .

[group("build")]
build-setup-image: (build-image "matverseny-setup" "containerfiles/setup.Containerfile")
[group("build")]
build-migration-image: (build-image "matverseny-migration" "containerfiles/migration.Containerfile")
[group("build")]
build-backend-image: (build-image "matverseny-backend" "Containerfile")

[group("build")]
build-images: build-setup-image build-migration-image build-backend-image

[private]
push-image NAME:
  podman push {{image_base}}/{{NAME}}:{{image_tag}}

[group("push")]
push-setup-image: (push-image "matverseny-setup")
[group("push")]
push-migration-image: (push-image "matverseny-migration")
[group("push")]
push-backend-image: (push-image "matverseny-backend")

[group("push")]
push-images: push-setup-image push-migration-image push-backend-image

workspace(name = "avocadocx")

## Dependencies:
load("//third_party:repos.bzl", "repos")

repos()

load("//third_party:deps.bzl", "deps")

deps()

## Toolchains:
load("@com_grail_bazel_toolchain//toolchain:rules.bzl", "llvm_toolchain")

llvm_toolchain(
    name = "llvm_toolchain",
    extra_targets = [
        "wasm32-unknown-wasi",
    ],
    llvm_version = "10.0.0",
)

load("@llvm_toolchain//:toolchains.bzl", "llvm_register_toolchains")

llvm_register_toolchains()

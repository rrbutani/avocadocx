"""Direct deps used in this project."""

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

RULES_GO_VER = "0.28.0"
RULES_GO_SHA = "8e968b5fcea1d2d64071872b12737bbb5514524ee5f0a4f54f5920266c261acb"

GAZELLE_VER = "0.23.0"
GAZELLE_SHA = "62ca106be173579c0a167deb23358fdfe71ffa1e4cfdddf5582af26520f1c66f"

PROTOBUF_VER = "3.17.3"
PROTOBUF_SHA = "c6003e1d2e7fefa78a3039f19f383b4f3a61e81be8c19356f85b6461998ad3db"

BAZEL_BUILD_TOOLS_VER = "4.0.1"
BAZEL_BUILD_TOOLS_SHA = "932160d5694e688cb7a05ac38efba4b9a90470c75f39716d85fb1d2f95eec96d"

# Using a fork, for now.
LLVM_TOOLCHAIN_VER = "a0f0eb4c1a8fdd15a7a0e4ff780dc6b846965d72"
LLVM_TOOLCHAIN_SHA = "0ed522b96c4b81638796357dd4c71ba8e15f5b99723b9d7f080eac4374557a68"

BAZEL_SKYLIB_VER = "1.0.3"
BAZEL_SKYLIB_SHA = "1c531376ac7e5a180e0237938a2536de0c54d93f5c278634818e0efc952dd56c"

LLVM_VER = "llvmorg-13.0.0-rc1"
LLVM_SHA = "7697c5716914c281ef7e656f3f1bb2fb4004c3d7ae223be83f9d05c01c02f81b"

def conditionally(excludes, next_func, **kwargs):
    if kwargs["name"] not in excludes:
        next_func(**kwargs)

def repos(excludes = []):
    """Sets up the direct dependencies used in this project.

    Args:
      excludes: repositories to skip setting up
    """

    # Needed for `buildifier`.
    conditionally(
        excludes,
        http_archive,
        name = "io_bazel_rules_go",
        sha256 = RULES_GO_SHA,
        canonical_id = RULES_GO_VER,
        urls = [
            "https://mirror.bazel.build/github.com/bazelbuild/rules_go/releases/download/v{ver}/rules_go-v{ver}.zip".format(ver = RULES_GO_VER),
            "https://github.com/bazelbuild/rules_go/releases/download/v{ver}/rules_go-v{ver}.zip".format(ver = RULES_GO_VER),
        ],
    )

    # Needed for `buildifier`.
    conditionally(
        excludes,
        http_archive,
        name = "bazel_gazelle",
        sha256 = GAZELLE_SHA,
        canonical_id = GAZELLE_VER,
        urls = [
            "https://mirror.bazel.build/github.com/bazelbuild/bazel-gazelle/releases/download/v{ver}/bazel-gazelle-v{ver}.tar.gz".format(ver = GAZELLE_VER),
            "https://github.com/bazelbuild/bazel-gazelle/releases/download/v{ver}/bazel-gazelle-v{ver}.tar.gz".format(ver = GAZELLE_VER),
        ],
    )

    # Needed for `buildifier`.
    conditionally(
        excludes,
        http_archive,
        name = "com_google_protobuf",
        sha256 = PROTOBUF_SHA,
        canonical_id = PROTOBUF_VER,
        strip_prefix = "protobuf-{ver}".format(ver = PROTOBUF_VER),
        urls = [
            "https://mirror.bazel.build/github.com/protocolbuffers/protobuf/archive/v{ver}.tar.gz".format(ver = PROTOBUF_VER),
            "https://github.com/protocolbuffers/protobuf/archive/v{ver}.tar.gz".format(ver = PROTOBUF_VER),
        ],
    )

    conditionally(
        excludes,
        http_archive,
        name = "com_github_bazelbuild_buildtools",
        sha256 = BAZEL_BUILD_TOOLS_SHA,
        canonical_id = BAZEL_BUILD_TOOLS_VER,
        strip_prefix = "buildtools-{ver}".format(ver = BAZEL_BUILD_TOOLS_VER),
        url = "https://github.com/bazelbuild/buildtools/archive/{ver}.zip".format(ver = BAZEL_BUILD_TOOLS_VER),
    )

    conditionally(
        excludes,
        http_archive,
        name = "com_grail_bazel_toolchain",
        sha256 = LLVM_TOOLCHAIN_SHA,
        canonical_id = LLVM_TOOLCHAIN_VER,
        strip_prefix = "bazel-toolchain-{ver}".format(ver = LLVM_TOOLCHAIN_VER),
        # NOTE(build): this is a fork, for now.
        url = "https://github.com/rrbutani/bazel-toolchain/archive/{ver}.tar.gz".format(ver = LLVM_TOOLCHAIN_VER),
    )

    # Needed for `llvm`:
    conditionally(
        excludes,
        http_archive,
        name = "bazel_skylib",
        sha256 = BAZEL_SKYLIB_SHA,
        canonical_id = BAZEL_SKYLIB_VER,
        urls = [
            "https://mirror.bazel.build/github.com/bazelbuild/bazel-skylib/releases/download/{version}/bazel-skylib-{version}.tar.gz".format(version = BAZEL_SKYLIB_VER),
            "https://github.com/bazelbuild/bazel-skylib/releases/download/{version}/bazel-skylib-{version}.tar.gz".format(version = BAZEL_SKYLIB_VER),
        ],
    )

    # As noted in the `llvm-project` examples, it's unfortunate that we grab
    # this repo twice but not ultimately significant due to caching.
    #
    # See: https://github.com/llvm/llvm-project/blob/d6974c010878cae1df5b27067230ee5dcbc63342/utils/bazel/examples/http_archive/WORKSPACE#L30-L31
    conditionally(
        excludes,
        http_archive,
        name = "llvm-project-raw",
        build_file_content = "#",
        sha256 = LLVM_SHA,
        canonical_id = LLVM_VER,
        strip_prefix = "llvm-project-{ver}".format(ver = LLVM_VER),
        url = "https://github.com/llvm/llvm-project/archive/{ver}.tar.gz".format(ver = LLVM_VER),
    )

    conditionally(
        excludes,
        http_archive,
        name = "llvm-bazel",
        sha256 = LLVM_SHA,
        canonical_id = LLVM_VER,
        strip_prefix = "llvm-project-{ver}/utils/bazel".format(ver = LLVM_VER),
        url = "https://github.com/llvm/llvm-project/archive/{ver}.tar.gz".format(ver = LLVM_VER),
    )

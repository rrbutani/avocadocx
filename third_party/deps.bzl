"""Setup for the direct deps used in this project."""

load("@io_bazel_rules_go//go:deps.bzl", "go_register_toolchains", "go_rules_dependencies")  # buildifier: disable=out-of-order-load
load("@bazel_gazelle//:deps.bzl", "gazelle_dependencies")  # buildifier: disable=out-of-order-load
load("@com_google_protobuf//:protobuf_deps.bzl", "protobuf_deps")  # buildifier: disable=out-of-order-load
load("@com_grail_bazel_toolchain//toolchain:deps.bzl", "bazel_toolchain_dependencies")
load("@bazel_skylib//:workspace.bzl", "bazel_skylib_workspace")  # buildifier: disable=out-of-order-load
load("@llvm-bazel//:configure.bzl", "llvm_configure", "llvm_disable_optional_support_deps")  # buildifier: disable=out-of-order-load

def closure(func, *args, **kwargs):
    return (func, args, kwargs)

# buildifier: disable=unsorted-dict-items
SETUP_FUNCTIONS = {
    "io_bazel_rules_go": [
        go_rules_dependencies,
        closure(go_register_toolchains, version = "1.16.5"),
    ],
    "bazel_gazelle": gazelle_dependencies,
    "com_google_protobuf": protobuf_deps,
    "com_grail_bazel_toolchain": bazel_toolchain_dependencies,
    "bazel_skylib": bazel_skylib_workspace,
    "llvm-project": [
        closure(
            llvm_configure,
            name = "llvm-project",
            src_path = ".",
            src_workspace = "@llvm-project-raw//:WORKSPACE",
        ),
        # Disables `zlib` and `terminfo` deps.
        llvm_disable_optional_support_deps,
    ],
}

def call_func(func_spec):
    if type(func_spec) == "tuple":
        func, args, kwargs = func_spec
        func(*args, **kwargs)
    else:
        func_spec()

def deps(excludes = []):
    """Does setup *for* the direct dependencies used in this project.

    Args:
      excludes: repositories to skip doing setup for
    """

    for repo, setup_functions in SETUP_FUNCTIONS.items():
        if repo not in excludes:
            if type(setup_functions) == "list":
                for func_spec in setup_functions:
                    call_func(func_spec)
            else:
                call_func(setup_functions)

load("@com_github_bazelbuild_buildtools//buildifier:def.bzl", "buildifier", "buildifier_test")

## Platforms

platform(
    name = "wasm",
    constraint_values = [
        "@platforms//cpu:wasm32",
        "@platforms//os:wasi",
    ],
)

## Starlark:

STARLARK_SRCS = [
    "BUILD",
    "WORKSPACE",
    "//third_party:build_files",
]

# Full list is here: https://github.com/bazelbuild/buildtools/blob/master/WARNINGS.md
BUILDIFIER_LINT_WARNINGS = [
    "all",
]

buildifier(
    name = "starlark-fix",
    lint_mode = "fix",
    lint_warnings = BUILDIFIER_LINT_WARNINGS,
    mode = "fix",
    verbose = True,
)

buildifier_test(
    name = "starlark-lint",
    srcs = STARLARK_SRCS,
    # diff_command = "colordiff -C 3",
    lint_mode = "warn",
    lint_warnings = BUILDIFIER_LINT_WARNINGS,
    mode = "diff",
    verbose = True,
)

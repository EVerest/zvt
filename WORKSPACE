load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "814680e1ab535f799fd10e8739ddca901351ceb4d2d86dd8126c22d36e9fcbd9",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.29.0/rules_rust-v0.29.0.tar.gz"],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains()

load("@rules_rust//crate_universe:defs.bzl", "crate", "crates_repository")

crates_repository(
    name = "crate_index",
    cargo_lockfile = "//:Cargo.lock",
    isolated = False,
    manifests = [
        "//:Cargo.toml",
        "//zvt:Cargo.toml",
        "//zvt_builder:Cargo.toml",
        "//zvt_cli:Cargo.toml",
        "//zvt_derive:Cargo.toml",
    ],
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()

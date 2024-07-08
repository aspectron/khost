use crate::imports::*;

pub fn install(ctx: &Context) -> Result<()> {
    ctx.root()?;

    log::step("Updating aptitude repositories...")?;
    cmd!("apt", "update", "-y").run()?;
    log::step("Upgrading OS...")?;
    cmd!("apt", "upgrade", "-y").run()?;
    log::step("Installing prerequisites...")?;
    cmd(
        "apt",
        [
            "install",
            "-y",
            "curl",
            "git",
            "build-essential",
            "libssl-dev",
            "pkg-config",
            "protobuf-compiler",
            "libprotobuf-dev",
            "clang-format",
            "clang-tidy",
            "clang-tools",
            "clang",
            "clangd",
            "libc++-dev",
            "libc++1",
            "libc++abi-dev",
            "libc++abi1",
            "libclang-dev",
            "libclang1",
            "liblldb-dev",
            "libllvm-ocaml-dev",
            "libomp-dev",
            "libomp5",
            "lld",
            "lldb",
            "llvm-dev",
            "llvm-runtime",
            "llvm",
            "python3-clang",
        ],
    )
    .run()?;

    Ok(())
}

pub fn detect() -> bool {
    let git_version = git::version();
    let protoc_version = protoc::version();
    let curl_version = cmd!("curl", "--version")
        .read()
        .ok()
        .map(|s| s.trim().to_string());
    let clang_version = cmd!("clang", "--version")
        .read()
        .ok()
        .map(|s| s.trim().to_string());

    git_version.is_some()
        && protoc_version.is_some()
        && curl_version.is_some()
        && clang_version.is_some()
}

pub mod protoc {
    use crate::imports::*;

    pub fn version() -> Option<String> {
        cmd!("protoc", "--version")
            .read()
            .ok()
            .map(|s| s.trim().to_string())
    }
}

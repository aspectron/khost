use crate::imports::*;

pub fn update(ctx: &Context) -> Result<()> {
    install(ctx)
}

pub fn install(ctx: &Context) -> Result<()> {
    log::remark(format!(
        "Upgrading {}...",
        ctx.system
            .long_os_version
            .as_ref()
            .unwrap_or(&"OS".to_string())
    ))?;

    step("Updating aptitude repositories...", || {
        sudo!("apt", "update", "-y").run()
    })?;

    step("Upgrading OS packages...", || {
        sudo!("apt", "upgrade", "-y").run()
    })?;

    let packages = [
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
    ];

    let len = packages.iter().map(|s| s.len()).max().unwrap();

    progress(packages.len(), "Installing prerequisites...", |progress| {
        for package in packages.iter() {
            progress.inc(1);
            progress.set_message(package.pad_to_width(len));

            sudo!("apt", "install", "-y", package).run()?;
        }

        Ok("Prerequisites installed successfully.")
    })?;

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

fn main() {
    // If it's a release-build, we tell the linker change the entrypoint.
    if cfg!(not(debug_assertions)) {
        link_winmain();
    }
}

fn link_winmain() {
    if cfg!(target_env = "msvc") {
        println!("cargo:rustc-link-arg-bins=/ENTRY:WinMainCRTStartup");
    } else {
        println!("cargo:rustc-link-arg-bins=-eWinMainCRTStartup");
    }
}

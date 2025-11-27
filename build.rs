use std::env;

fn main() {
    // Set up Python environment for PyO3
    // This helps PyO3 find the correct Python installation

    // Print cargo directives
    println!("cargo:rerun-if-env-changed=PYTHON_SYS_EXECUTABLE");

    // Optional: Add custom build logic here if needed
    // For example, checking if Spleeter is installed

    // On Windows, you might need to link additional libraries
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=gdi32");
    }
}

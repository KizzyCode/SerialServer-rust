use cc::Build;
use std::env::consts::FAMILY;

/// Select the platform specific source file
fn select_impl() -> &'static str {
    match FAMILY {
        "unix" => "src/serial/unix.c",
        family => panic!("Unsupported target OS family: {family}"),
    }
}

fn main() {
    // Build and link the helper shim
    Build::new().file(select_impl()).warnings_into_errors(true).compile("serial");
}

#![no_std]
#![no_main]

#[cfg(target_os = "macos")]
#[link(name = "System")]
unsafe extern "C" {}

#[cfg(target_os = "linux")]
#[link(name = "c")]
unsafe extern "C" {}

#[link(name = "ssl")]
unsafe extern "C" {}

#[link(name = "crypto")]
unsafe extern "C" {}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &::core::panic::PanicInfo) -> ! {
    volki::core::cli::print_error(&volki::vformat!("volki panic: {}", info));
    volki::core::volkiwithstds::process::exit(101);
}

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let cli = volki::core::cli::build_cli();
    if let Err(e) = cli.run() {
        volki::core::cli::print_cli_error(&e);
        volki::core::volkiwithstds::process::exit(1);
    }
    0
}

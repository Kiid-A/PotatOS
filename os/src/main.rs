#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

//use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE, INPUT_CONDVAR};
use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE};
extern crate alloc;

#[macro_use]
extern crate bitflags;

use fs::proc;
use log::*;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod lang_items;
mod logging;
mod mm;
mod net;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

use crate::drivers::chardev::CharDevice;
use crate::drivers::chardev::UART;

core::arch::global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

fn boot_screen() {
    let boot_screen = r#"
    @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    @@@@@@@@@@@@@@@@@@@#+-:.....:=*%@@@@@@@@
    @@@@@@@@@@@@@@@@#-   :--===-:.  .+@@@@@@
    @@@@@@@@@@@@@@%-  -+***********=:  +@@@@
    @@@@@@@@@@@@@*  =****+--+*+++++**+: :@@@
    @@@@@@@@@@@%-  -+==++++++**+*=-=+**: :@@
    @@@@@@@@#=.  :==-==---=****+*+++*+**. *@
    @@@@@@#-  :=+*+***+++++***********+=. =@
    @@@@@-  =***************+**=-=**+=--. +@
    @@@@: :**+==+*++***+=-=+*+*+=++=--=-  %@
    @@@+ .**++==+*******+=--+***+=---=-  *@@
    @@@: -**+*****+++****+=-+*+=---=-. .#@@@
    @@@: :=+***+++=-=*+++***+=---=-.  +@@@@@
    @@@= .--=++***+++****++=---=-.  +@@@@@@@
    @@@@. :=---===++++===----=-.  +@@@@@@@@@
    @@@@%: .-==-----------==-.  +@@@@@@@@@@@
    @@@@@@+   :-=========-:  .+@@@@@@@@@@@@@
    @@@@@@@@*:    .....    :*@@@@@@@@@@@@@@@
    @@@@@@@@@@@#*=--:--=*#@@@@@@@@@@@@@@@@@@
    @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    ______     _        _   _____ _____ 
    | ___ \   | |      | | |  _  /  ___|
    | |_/ /__ | |_ __ _| |_| | | \ `--. 
    |  __/ _ \| __/ _` | __| | | |`--. \
    | | | (_) | || (_| | |_\ \_/ /\__/ /
    \_|  \___/ \__\__,_|\__|\___/\____/      v0.1
                                      
"#;
    println!("{}", boot_screen);
}

use lazy_static::*;
use sync::UPIntrFreeCell;

lazy_static! {
    pub static ref DEV_NON_BLOCKING_ACCESS: UPIntrFreeCell<bool> =
        unsafe { UPIntrFreeCell::new(false) };
}

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    mm::init();
    UART.init();
    info!("KERN: init gpu");
    let _gpu = GPU_DEVICE.clone();
    info!("KERN: init keyboard");
    let _keyboard = KEYBOARD_DEVICE.clone();
    info!("KERN: init mouse");
    let _mouse = MOUSE_DEVICE.clone();
    info!("KERN: init trap");
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    board::device_init();
    fs::list_apps();
    proc::init_proc();
    boot_screen();
    task::add_initproc();
    *DEV_NON_BLOCKING_ACCESS.exclusive_access(file!(), line!()) = true;
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}

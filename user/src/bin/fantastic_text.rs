#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

macro_rules! color_text {
    ($text:expr, $color:expr) => {{
        format_args!("\x1b[{}m{}\x1b[0m", $color, $text)
    }};
}

#[no_mangle]
pub fn main() -> i32 {
    println!(
        "{}{}{}{}{} {}{}{}{}{}{}{}{}",
        color_text!("H", 31),
        color_text!("e", 32),
        color_text!("l", 33),
        color_text!("l", 34),
        color_text!("o", 35),
        color_text!("P", 36),
        color_text!("o", 37),
        color_text!("t", 90),
        color_text!("a", 91),
        color_text!("t", 92),
        color_text!("O", 93),
        color_text!("S", 94),
        color_text!("!", 95),
    );

    let text =
        "reguler \x1b[4munderline\x1b[24m \x1b[7mreverse\x1b[27m \x1b[9mstrikethrough\x1b[29m";
    println!("\x1b[47m{}\x1b[0m", color_text!(text, 30));
    for i in 31..38 {
        println!("{}", color_text!(text, i));
    }
    for i in 90..98 {
        println!("{}", color_text!(text, i));
    }
    0
}

extern crate libc;

extern {
    fn double_input(input: libc::c_int) -> libc::c_int;
}

fn main() {
    let input = 3;
    let output = unsafe { double_input(input) };
    println!("return to rust {} * 3 = {}", input, output);
}

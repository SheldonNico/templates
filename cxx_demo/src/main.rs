#[cxx::bridge(namespace = org::example)]
mod ffi {
    struct SharedThing {
        z: i32,
        y: Box<ThingR>,
        x: UniquePtr<ThingC>,
    }

    extern "C" {
        include!("src/demo.h");

        type ThingC;
        fn make_demo(appname: &str) -> UniquePtr<ThingC>;
        fn get_name(thing: &ThingC) -> &CxxString;
        fn do_thing(state: SharedThing);
    }

    extern "Rust" {
        type ThingR;
        fn print_r(r: &ThingR);
    }
}

#[link(name = "demo_c")]
extern {
    fn cool_function(i: std::os::raw::c_int, c: std::os::raw::c_char);
}

pub struct ThingR(usize);

fn print_r(r: &ThingR) {
    println!("called back with r={}", r.0);
}

fn main() {
    let x = ffi::make_demo("demo of cxx::bridge");
    println!("this is a {}", ffi::get_name(x.as_ref().unwrap()));

    ffi::do_thing(ffi::SharedThing {
        z: 222,
        y: Box::new(ThingR(333)),
        x,
    });

    unsafe {
        cool_function(1, 's' as i8);
    }
}

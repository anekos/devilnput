
extern crate argparse;
extern crate ioctl;
extern crate wraited_struct;


use std::fs::File;
use std::fmt::Display;
use std::os::unix::io::{AsRawFd, RawFd};
use argparse::{ArgumentParser, Store, StoreConst, Print};


#[repr(C)]
struct RawEventTime {
    pub seconds: u64,
    pub microseconds: u64
}

#[repr(C)]
struct RawInputEvent {
    pub time:  RawEventTime,
    pub kind:  u16,
    pub code:  u16,
    pub value: i32
}

#[derive(Copy, Clone)]
enum Format {
    TabSplit,
    Eval
}



macro_rules! puts_kvs_in_eval {
    ( $name:expr => $value:expr $(, $tname:expr => $tvalue:expr)* ) => {
        {
            print!("{}={}", $name, $value);
            $( print!(" {}={}", $tname, $tvalue); )*
            println!("");
        }
    }
}


macro_rules! puts_kvs_in_tab_split {
    ( $name:expr => $value:expr $(, $tname:expr => $tvalue:expr)* ) => {
        {
            print!("{}", $value);
            $( print!("\t{}", $tvalue); )*
            println!("");
        }
    }
}


macro_rules! puts_kvs {
    ( $format:expr $(,$name:expr => $value:expr)* ) => {
        match $format {
            Format::TabSplit => puts_kvs_in_eval!($($name => $value),*),
            Format::Eval => puts_kvs_in_tab_split!($($name => $value),*)
        }
    }
}



fn main() {
    use Format::*;

    let mut file = String::new();
    let mut format = Format::Eval;

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("panty and stocking");
        ap.refer(&mut format)
            .add_option(&["-e", "--eval"], StoreConst(Eval), "for eval (default)")
            .add_option(&["-t", "--tab-split"], StoreConst(TabSplit), "Tab split");
        ap.refer(&mut file).add_argument("Device file", Store, "/dev/input/*");
        ap.add_option(&["-v", "--version"], Print(env!("CARGO_PKG_VERSION").to_string()), "Show version");
        ap.parse_args_or_exit();
    }

    let mut file = File::open(file).expect("Could not open");
    let fd: RawFd = file.as_raw_fd();

    unsafe {
        ioctl::eviocgrab(fd, &1);
    }

    while let Ok(event) = wraited_struct::read::<RawInputEvent, File>(&mut file) {
        puts_kvs!(format, "time" => event.time.seconds, "kind" => event.kind, "code" => event.code, "value" => event.value);
    }
}

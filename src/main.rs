
extern crate argparse;
extern crate ioctl;
extern crate wraited_struct;
extern crate uinput_sys;

mod name_table;

use std::fs::File;
use std::collections::HashMap;
use std::os::unix::io::{AsRawFd, RawFd};
use argparse::{ArgumentParser, Store, StoreConst, Print};
use uinput_sys::*;

use name_table::*;


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
            Format::Eval => puts_kvs_in_eval!($($name => $value),*),
            Format::TabSplit => puts_kvs_in_tab_split!($($name => $value),*)
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

    let num2code = generate_code_name_table();
    let num2ev = generate_ev_name_table();
    let num2rel = generate_rel_name_table();

    let mut file = File::open(file).expect("Could not open");
    let fd: RawFd = file.as_raw_fd();

    unsafe {
        ioctl::eviocgrab(fd, &1);
    }

    while let Ok(event) = wraited_struct::read::<RawInputEvent, File>(&mut file) {
        let code_name = name(event.code, &num2code);
        let kind_name = name(event.kind, &num2ev);
        match event.kind as i32 {
            EV_SYN | EV_MSC => (),
            EV_KEY =>
                puts_kvs!(
                    format,
                    "time" => event.time.seconds,
                    "kind" => kind_name,
                    "code" => code_name,
                    "value" => event.value),
            EV_REL => {
                let rel_name = name(event.code, &num2rel);
                puts_kvs!(
                    format,
                    "time" => event.time.seconds,
                    "kind" => kind_name,
                    "code" => rel_name,
                    "value" => event.value);
            }
            _ =>
                puts_kvs!(
                    format,
                    "time" => event.time.seconds,
                    "kind" => kind_name,
                    "code" => code_name,
                    "value" => event.value),
        }
    }
}


fn name(num: u16, table: &HashMap<u16, String>) -> String {
    table.get(&num).unwrap_or(&format!("{}", num)).to_owned()
}

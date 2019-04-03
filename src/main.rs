
extern crate argparse;
extern crate evdev_rs;
extern crate uinput_sys;
extern crate wraited_struct;

use std::fs::File;
use std::collections::HashMap;

use argparse::{ArgumentParser, Store, StoreConst, StoreTrue, Print};
use evdev_rs::{Device, GrabMode};
use uinput_sys::*;

mod name_table;

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
    List,
    Eval
}



macro_rules! puts_kvs_in_eval {
    ( $delimiter:expr, $name:expr => $value:expr $(, $tname:expr => $tvalue:expr)* ) => {
        {
            print!("{}={}", $name, $value);
            $( print!("{}{}={}", $delimiter, $tname, $tvalue); )*
            println!();
        }
    }
}


macro_rules! puts_kvs_in_list {
    ( $delimiter:expr, $name:expr => $value:expr $(, $tname:expr => $tvalue:expr)* ) => {
        {
            print!("{}", $value);
            $( print!("{}{}", $delimiter, $tvalue); )*
            println!();
        }
    }
}


macro_rules! puts_kvs {
    ( $delimiter:expr, $format:expr $(,$name:expr => $value:expr)* ) => {
        match $format {
            Format::Eval => puts_kvs_in_eval!($delimiter, $($name => $value),*),
            Format::List => puts_kvs_in_list!($delimiter, $($name => $value),*)
        }
    }
}



fn main() {
    use Format::*;

    let mut file = String::new();
    let mut format = Format::Eval;
    let mut named = false;
    let mut delimiter = " ".to_owned();
    let mut grab = false;

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("DEViLnput");
        ap.refer(&mut format)
            .add_option(&["-e", "--eval"], StoreConst(Eval), "For eval (default)")
            .add_option(&["-l", "--list"], StoreConst(List), "List");
        ap.refer(&mut grab)
            .add_option(&["-g", "--grab"], StoreTrue, "Grab device");
        ap.refer(&mut named)
            .add_option(&["-n", "--named"], StoreTrue, "Show named values");
        ap.refer(&mut delimiter).add_option(&["-d", "--delimiter"], Store, "Item delimiter");
        ap.refer(&mut file)
            .add_argument("Device file", Store, "/dev/input/*")
            .required();
        ap.add_option(&["-v", "--version"], Print(env!("CARGO_PKG_VERSION").to_string()), "Show version");
        ap.parse_args_or_exit();
    }

    let num2code = generate_code_name_table();
    let num2ev = generate_ev_name_table();
    let num2rel = generate_rel_name_table();
    let num2abs = generate_abs_name_table();

    let mut file = File::open(file).expect("Could not open");

    let mut device = Device::new_from_fd(&file).unwrap();
    if grab {
        device.grab(GrabMode::Grab).unwrap();
    }

    while let Ok(event) = unsafe { wraited_struct::read::<RawInputEvent, File>(&mut file) } {
        let kind_name = name(named, false, event.kind, &num2ev);
        match i32::from(event.kind) {
            EV_SYN | EV_MSC => (),
            EV_KEY => {
                let code_name = name(named, true, event.code, &num2code);
                puts_kvs!(
                    delimiter,
                    format,
                    "time" => event.time.seconds,
                    "kind" => kind_name,
                    "code" => code_name,
                    "value" => event.value);
            }
            EV_REL => {
                let rel_name = name(named, true, event.code, &num2rel);
                puts_kvs!(
                    delimiter,
                    format,
                    "time" => event.time.seconds,
                    "kind" => kind_name,
                    "code" => rel_name,
                    "value" => event.value);
            }
            EV_ABS => {
                let abs_name = name(named, true, event.code, &num2abs);
                puts_kvs!(
                    delimiter,
                    format,
                    "time" => event.time.seconds,
                    "kind" => kind_name,
                    "code" => abs_name,
                    "value" => event.value);
            }
            _ =>
                puts_kvs!(
                    delimiter,
                    format,
                    "time" => event.time.seconds,
                    "kind" => kind_name,
                    "code" => event.code,
                    "value" => event.value),
        }
    }
}


fn name(enabled: bool, padding: bool, num: u16, table: &HashMap<u16, String>) -> String {
    if enabled {
        let result = table.get(&num).unwrap_or(&format!("{}", num)).to_owned();
        if padding {
            pad(&result)
        } else {
            result
        }
    } else {
        format!("{}", num)
    }
}


fn pad(s: &str) -> String {
    format!("{:width$}", s, width = MAX_NAME_SIZE)
}

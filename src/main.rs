use std::{env,path,process};

extern crate byteorder;
extern crate getopts;
extern crate libudev;
#[macro_use] extern crate nix;

mod da2013;

fn usage(progname: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", progname);
    print!("{}", opts.usage(&brief));
}

fn get_hidraw_node() -> Option<path::PathBuf> {
    let udev = libudev::Context::new().expect("Failed to create udev context");
    let mut enumerator = libudev::Enumerator::new(&udev)
        .expect("Failed to obtain udev enumerator");

    enumerator.match_subsystem("hidraw").expect("Failed to match hidraw subsystem");

    for device in enumerator.scan_devices().expect("Failed to scan devices") {
        if let Some(usb_parent) = device.parent_with_subsystem_devtype("usb", "usb_device") {
            if usb_parent.attribute_value("idVendor")
                .expect("Failed to get vendor ID") == "1532" &&
                usb_parent.attribute_value("idProduct")
                .expect("Failed to get product ID") == "0037" {
                return Some(device.devnode().expect("Failed to find /dev node").to_owned());
            }
        }
    }

    None
}

fn boolarg(a: String, argname: &str) -> bool {
    match a.to_lowercase().as_str() {
        "on" | "true" | "1" | "enabled" => true,
        "off" | "false" | "0" | "disabled" => false,
        _ => {
            println!("Invalid boolean for {}", argname);
            process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let progname = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optopt("d", "device", "hidraw node of the mouse", "DEV");
    opts.optopt("r", "dpi", "Sensor resolution (100-6400 DPI in increments of 100)", "RES/X,Y");
    opts.optopt("f", "freq", "Polling rate (125, 500 or 1000 Hz)", "HZ");
    opts.optopt("l", "logo", "Logo LED", "on/off");
    opts.optopt("w", "wheel", "Scroll wheel LED", "on/off");
    opts.optflag("h", "help", "Print help and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f.to_string());
            usage(&progname, opts);
            process::exit(1);
        }
    };

    // Show help when loose arguments are passed
    if matches.opt_present("h") || !matches.free.is_empty() {
        usage(&progname, opts);
        return;
    }

    // TODO I'm sure this can be improved somehow
    let dev = match (matches.opt_str("d"), get_hidraw_node()) {
        (Some(d), _) => path::PathBuf::from(d),
        (None, Some(d)) => d,
        (None, None) => {
            println!("Can't detect the device with udev, make sure it's plugged in, \
                     or specify -d");
            usage(&progname, opts);
            process::exit(1);
        }
    };

    let dpi = matches.opt_str("r").map(|s| match s.parse::<i32>() {
        Ok(r) => {
            if r < 100 || r > 6400 || (r % 100 != 0) {
                println!("Invalid resolution value");
                process::exit(1);
            }
            r
        }
        Err(_) => {
            println!("Resolution is not a valid integer");
            process::exit(1);
        }
    });

    let freq = matches.opt_str("f").map(|s| {
        match s.as_str() {
            "125" => da2013::Freq::F125,
            "500" => da2013::Freq::F500,
            "1000" => da2013::Freq::F1000,
            _ => {
                println!("Polling rate must be one of 125, 500 or 1000 Hz");
                process::exit(1);
            }
        }
    });

    let led_logo = matches.opt_str("l").map(|s| boolarg(s, "logo LED"));
    let led_wheel = matches.opt_str("w").map(|s| boolarg(s, "wheel LED"));

    let mouse = match da2013::Da2013::open(dev) {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to open device: {}", e.to_string());
            process::exit(1);
        }
    };

    if let Some(res) = dpi {
        mouse.set_res(res);
    }
    if let Some(freq) = freq {
        mouse.set_freq(freq);
    }
    if let Some(state) = led_logo {
        mouse.set_led(da2013::Led::Logo, state);
    }
    if let Some(state) = led_wheel {
        mouse.set_led(da2013::Led::Wheel, state);
    }
}

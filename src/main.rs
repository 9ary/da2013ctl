extern crate getopts;
extern crate libudev;

fn usage(progname: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", progname);
    print!("{}", opts.usage(&brief));
}

fn get_hidraw_node() -> Option<std::path::PathBuf> {
    let udev = libudev::Context::new().unwrap();
    let mut enumerator = libudev::Enumerator::new(&udev).unwrap();

    enumerator.match_subsystem("hidraw").unwrap();

    for device in enumerator.scan_devices().unwrap() {
        if let Some(usb_parent) = device.parent_with_subsystem_devtype("usb", "usb_device") {
            if usb_parent.attribute_value("idVendor").unwrap() == "1532" &&
                usb_parent.attribute_value("idProduct").unwrap() == "0037" {
                return Some(device.devnode().unwrap().to_owned());
            }
        }
    }

    None
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
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
            return;
        }
    };

    // Show help when loose arguments are passed
    if matches.opt_present("h") || !matches.free.is_empty() {
        usage(&progname, opts);
        return;
    }

    // TODO I'm sure this can be improved somehow
    let dev = match (matches.opt_str("d"), get_hidraw_node()) {
        (Some(d), _) => std::path::PathBuf::from(d),
        (None, Some(d)) => d,
        (None, None) => {
            println!("Can't detect the device with udev, make sure it's plugged in, \
                     or specify -d");
            usage(&progname, opts);
            return;
        }
    };

    let freq = matches.opt_str("f").map(|s| {
        match s.as_str() {
            "125" => 125,
            "500" => 500,
            "1000" => 1000,
            _ => {
                println!("Polling rate must be one of 125, 500 or 1000 Hz");
                std::process::exit(1);
            }
        }
    });

    println!("Using device {}", dev.to_str().unwrap());
}

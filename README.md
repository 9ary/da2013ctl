# da2013ctl
A simpler alternative to [razercfg](http://bues.ch/cms/hacking/razercfg.html),
tailored specifically for the DeathAdder 2013 for Synapse-like seamless setting
changes.

## Compiling
```
$ make
```

## Installing
Create the group `razer` and add yourself to it then:
```
# make install
# udevadm control --reload
# udevadm trigger
```

## Dependencies
Only `libudev` is required. This will only work on Linux kernels since it relies
on the `hidraw` API.

## Usage
See `da2013ctl --help` for more info.

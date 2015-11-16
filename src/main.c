#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <argp.h>
#include <libudev.h>
#include "da2013.h"

struct args
{
    char *dev;
    int dpix, dpiy;
    int freq;
    int led_logo;
    int led_wheel;
};

static struct args args = {.dev = NULL,
                           .dpix = 0,
                           .dpiy = 0,
                           .freq = 0,
                           .led_logo = -1,
                           .led_wheel = -1};

static struct argp_option options[] = {
    {"dpi", 'r', "RES", 0, "Set sensor resolution to RES", 0},
    {"freq", 'f', "HZ", 0, "Set polling frequency to HZ", 0},
    {"logo", 'l', "STATE", 0, "Turn logo LED on or off", 0},
    {"wheel", 'w', "STATE", 0, "Turn scroll wheel LED on of off", 0},
    {"device", 'd', "DEV", 0, "hidraw device to use", 0},
    {NULL, 0, NULL, 0, NULL, 0}};

static const char description[] = "Apply Razer DeatAdder 2013 firmware settings";

static int boolstr(char *str)
{
    if (strcmp(str, "on") == 0)
        return 1;

    if (strcmp(str, "off") == 0)
        return 0;

    return -1;
}

static error_t parse_opt(int key, char *arg, struct argp_state *state)
{
    struct args *args = state->input;
    int i;

    switch (key)
    {
        case 'r':
            i = strtol(arg, NULL, 10);
            args->dpix = i;
            args->dpiy = i;
            if (i < 100 || i > 6400)
                argp_error(state, "%s: not a number or out of range", arg);
            break;

        case 'f':
            i = strtol(arg, NULL, 10);
            args->freq = i;
            if (i < 125 || i > 1000)
                argp_error(state, "%s: not a number or out of range", arg);
            break;

        case 'l':
            i = boolstr(arg);
            args->led_logo = i;
            if (i < 0)
                argp_error(state, "LED state must be on/off");
            break;

        case 'w':
            i = boolstr(arg);
            args->led_wheel = i;
            if (i < 0)
                argp_error(state, "LED state must be on/off");
            break;

        case 'd':
            args->dev = arg;
            break;

        default:
            return ARGP_ERR_UNKNOWN;
    };

    return 0;
}

static struct argp argp = {options, parse_opt, NULL, description, NULL, NULL, NULL};

int main(int argc, char **argv)
{
    argp_parse(&argp, argc, argv, 0, NULL, &args);

    if (!args.dev)
    {
        struct udev *udev;
        struct udev_enumerate *udev_enum;
        struct udev_list_entry *devs, *entry;
        struct udev_device *dev;
        const char *path, *node;

        udev = udev_new();
        if (!udev)
        {
            printf("Unable to contact udev\n");
            return 1;
        }

        udev_enum = udev_enumerate_new(udev);
        udev_enumerate_add_match_subsystem(udev_enum, "hidraw");
        udev_enumerate_scan_devices(udev_enum);
        devs = udev_enumerate_get_list_entry(udev_enum);

        udev_list_entry_foreach(entry, devs)
        {
            path = udev_list_entry_get_name(entry);
            dev = udev_device_new_from_syspath(udev, path);
            node = udev_device_get_devnode(dev);
            dev = udev_device_get_parent_with_subsystem_devtype(dev, "usb", "usb_device");
            if (!dev)
                continue;

            const char *vendor = udev_device_get_sysattr_value(dev, "idVendor");
            const char *device = udev_device_get_sysattr_value(dev, "idProduct");
            if (strcmp(vendor, "1532") == 0 && strcmp(device, "0037") == 0)
            {
                args.dev = malloc(strlen(node) + 1);
                strcpy(args.dev, node);
                udev_device_unref(dev);
                break;
            }

            udev_device_unref(dev);
        }

        udev_enumerate_unref(udev_enum);
        udev_unref(udev);
    }

    int fd;

    fd = open(args.dev, O_RDWR | O_NONBLOCK);

    if (fd < 0)
    {
        perror("Unable to open device");
        exit(1);
    }

    if (args.freq)
    {
        da2013_set_freq(args.freq);
        da2013_cmd_send(fd);
    }

    if (args.led_logo >= 0)
    {
        da2013_set_led_logo(args.led_logo);
        da2013_cmd_send(fd);
    }

    if (args.led_wheel >= 0)
    {
        da2013_set_led_wheel(args.led_wheel);
        da2013_cmd_send(fd);
    }

    if (args.dpix && args.dpiy)
    {
        da2013_set_dpi(args.dpix, args.dpiy);
        da2013_cmd_send(fd);
    }

    close(fd);
    return 0;
}

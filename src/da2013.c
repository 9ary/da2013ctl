#include <stdint.h>
#include <string.h>
#include <stdio.h>
#include <linux/input.h>
#include <linux/hidraw.h>
#include "da2013.h"
#include "utility.h"

#define DA2013_CMD_SET 0x0300
#define DA2013_REQ_DPI 0x0104
#define DA2013_REQ_LED 0x0003
#define DA2013_REQ_FREQ 0x0500

struct __attribute__((__packed__)) cmd
{
    uint8_t report;
    uint8_t status;
    uint8_t pad0[3];
    le16_t command;
    le16_t request;
    le16_t arg0;
    le16_t arg1;
    uint8_t pad1[76];
    uint8_t footer;
    uint8_t pad2[1];
};

static struct cmd da2013_cmd;
static struct cmd da2013_ret;

static void da2013_cmd_setup(uint16_t cmd, uint16_t req, uint16_t arg0,
                             uint16_t arg1, uint8_t foot)
{
    memset(&da2013_cmd, 0, sizeof(struct cmd));

    da2013_cmd.command = to_le16(cmd);
    da2013_cmd.request = to_le16(req);
    da2013_cmd.arg0 = to_le16(arg0);
    da2013_cmd.arg1 = to_le16(arg1);
    da2013_cmd.footer = foot;

    da2013_ret.report = 0;
}

void da2013_cmd_send(int fd)
{
    int res;

    // Pretty unreliable unless you do it a few times
    for (int i = 0; i < 3; i++)
    {
        res = ioctl(fd, HIDIOCSFEATURE(91), (char *) &da2013_cmd);
        if (res < 0)
            perror("HIDIOCSFEATURE");
        else
        {
            res = ioctl(fd, HIDIOCGFEATURE(91), (char *) &da2013_ret);
            if (res < 0)
                perror("HIDIOCGFEATURE");
        }
    }
}

void da2013_set_dpi(int x, int y)
{
    uint8_t xc = (x / 100 - 1) * 4;
    uint8_t yc = (y / 100 - 1) * 4;

    da2013_cmd_setup(DA2013_CMD_SET, DA2013_REQ_DPI, xc | (yc << 8), 0, 0x06);
}

void da2013_set_led_wheel(int value)
{
    uint16_t arg1;
    uint8_t foot;

    if (value)
    {
        arg1 = 1;
        foot = 0x01;
    }
    else
    {
        arg1 = 0;
        foot = 0x00;
    }

    da2013_cmd_setup(DA2013_CMD_SET, DA2013_REQ_LED, 0x0101, arg1, foot);
}

void da2013_set_led_logo(int value)
{
    uint16_t arg1;
    uint8_t foot;

    if (value)
    {
        arg1 = 1;
        foot = 0x04;
    }
    else
    {
        arg1 = 0;
        foot = 0x05;
    }

    da2013_cmd_setup(DA2013_CMD_SET, DA2013_REQ_LED, 0x0401, arg1, foot);
}

void da2013_set_freq(int value)
{
    uint16_t arg0;
    uint8_t foot;

    if (value >= 1000)
    {
        arg0 = 1;
        foot = 0x05;
    }
    else if (value >= 500)
    {
        arg0 = 2;
        foot = 0x06;
    }
    // Default to 125 Hz
    else
    {
        arg0 = 8;
        foot = 0x0C;
    }

    da2013_cmd_setup(DA2013_CMD_SET, DA2013_REQ_FREQ, arg0, 0, foot);
}

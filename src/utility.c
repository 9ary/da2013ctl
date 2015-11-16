#include <utility.h>

le16_t to_le16(uint16_t x)
{
    le16_t ret;
    uint8_t *ret8 = (uint8_t *) &ret;

    ret8[0] = x & 0xFF;
    ret8[1] = (x >> 8) & 0xFF;

    return ret;
}

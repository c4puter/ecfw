/*
 * The MIT License (MIT)
 * Copyright (c) 2016 Chris Pavlina
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
 * DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
 * OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE
 * OR OTHER DEALINGS IN THE SOFTWARE.
 */

#include <asf/drivers/usart/usart.h>
#include <asf/services/clock/sysclk.h>
#include "conf_usart.h"
#include "usart.h"

#define ALL_INTERRUPT_MASK  0xffffffff

void ec_usart_init(void)
{
    static const sam_usart_opt_t usart_settings = {
        USART_SERIAL_BAUDRATE,
        USART_SERIAL_CHAR_LENGTH,
        USART_SERIAL_PARITY,
        USART_SERIAL_STOP_BIT,
        US_MR_CHMODE_NORMAL,
        .irda_filter = 0,
    };

    sysclk_enable_peripheral_clock(USART_SERIAL_ID);
    usart_init_rs232(USART_SERIAL, &usart_settings,
            sysclk_get_peripheral_bus_hz(USART_SERIAL));
    usart_enable_tx(USART_SERIAL);
    usart_enable_rx(USART_SERIAL);
}

void ec_usart_putc(char c)
{
    usart_putchar(USART_SERIAL, c);
}

char ec_usart_getc()
{
    int c = 0;
    usart_getchar(USART_SERIAL, &c);
    if (c > 0 && c <= 255) {
        return (char) c;
    }
}

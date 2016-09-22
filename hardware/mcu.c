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

#include <stdio.h>
#include <asf/boards/board.h>
#include <asf/services/ioport/ioport.h>
#include <asf/services/clock/sysclk.h>
#include "mcu.h"

#define LED_GPIO IOPORT_CREATE_PIN(PIOC, 0)
//#define LED_GPIO IOPORT_CREATE_PIN(PIOC, 17)

#define UART0_TX_PIN IOPORT_CREATE_PIN(PIOA, 22)
#define UART0_RX_PIN IOPORT_CREATE_PIN(PIOA, 21)

void mcu_init(void)
{
    WDT->WDT_MR = WDT_MR_WDDIS;
    sysclk_init();
    NVIC_SetPriorityGrouping(0);
    ioport_init();
}

void board_init(void)
{
    ioport_set_pin_dir(LED_GPIO, IOPORT_DIR_OUTPUT);
    ioport_enable_pin(LED_GPIO);

    ioport_set_pin_mode(UART0_TX_PIN, IOPORT_MODE_MUX_A);
    ioport_disable_pin(UART0_TX_PIN);
    ioport_set_pin_mode(UART0_RX_PIN, IOPORT_MODE_MUX_A);
    ioport_disable_pin(UART0_RX_PIN);
}

void do_toggle_led(void)
{
    ioport_toggle_pin_level(LED_GPIO);
}

/*
 * On hard fault, this prepares an array of register values read from the stack
 * and calls hard_fault_printer. The values are:
 * {r0, r1, r2, r3, r12, lr, pc, psr}
 */
__attribute__((naked))
void HardFault_Handler(void)
{
    __asm volatile
        (
         " tst lr, #4                                                \n"
         " ite eq                                                    \n"
         " mrseq r0, msp                                             \n"
         " mrsne r0, psp                                             \n"
         " ldr r1, [r0, #24]                                         \n"
         " ldr r2, hard_fault_printer_const                          \n"
         " bx r2                                                     \n"
         " hard_fault_printer_const: .word hard_fault_printer        \n"
        );
}

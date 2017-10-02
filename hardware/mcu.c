/*
 * c4puter embedded controller firmware
 * Copyright (C) 2017 Chris Pavlina
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

#include <stdio.h>
#include <asf/boards/board.h>
#include <asf/services/ioport/ioport.h>
#include <asf/services/clock/sysclk.h>
#include <asf/drivers/spi/spi.h>
#include <FreeRTOS.h>
#include "mcu.h"

#define RS232_TX IOPORT_CREATE_PIN(PIOA, 22)

void mcu_init(void)
{
    WDT->WDT_MR = WDT_MR_WDDIS;
    sysclk_init();

    ioport_init();

    // Configure RS232 tx early, for debug output
    ioport_set_pin_mode(RS232_TX, IOPORT_MODE_MUX_A);
    ioport_disable_pin(RS232_TX);

    sysclk_enable_peripheral_clock(ID_TWI0);
    sysclk_enable_peripheral_clock(ID_USART1);
}


extern uint32_t _sstack;
extern uint32_t _estack;
register uint32_t *sp __asm__("sp");


void write_stack_canaries(void)
{
    for (uint32_t *i = &_sstack; i < sp; ++i) {
        *i = 0xdeadbeef;
    }
}

uint32_t get_stack_unused(void)
{
    uint32_t *i;
    for (i = &_sstack; i < &_estack; ++i) {
        if (*i != 0xdeadbeef)
            break;
    }

    return (uint32_t)i - (uint32_t)&_sstack;
}

unsigned int mcu_get_peripheral_hz(void)
{
    return sysclk_get_peripheral_hz();
}

bool mcu_get_pin_level(unsigned int pin)
{
    // Fucking Atmel force-inlines this, so un-inline it so Rust can find it
    return ioport_get_pin_level(pin);
}

void mcu_set_pin_level(unsigned int pin, bool value)
{
    // Fucking Atmel force-inlines this, so un-inline it so Rust can find it
    ioport_set_pin_level(pin, value);
}

void mcu_init_pin(unsigned int pin, unsigned int mode_mask, bool default_value)
{
    // Mask: see definitions in gpio.rs
    bool is_periph = mode_mask & 0x80000000u;
    bool is_output = mode_mask & 0x40000000u;
    uint32_t ioport_mode = mode_mask & 0xffffu;

    ioport_set_pin_mode(pin, ioport_mode);

    if (is_periph) {
        ioport_disable_pin(pin);
    } else {
        if (is_output) {
            ioport_set_pin_level(pin, default_value);
            ioport_set_pin_dir(pin, IOPORT_DIR_OUTPUT);
        } else {
            ioport_set_pin_dir(pin, IOPORT_DIR_INPUT);
        }
        ioport_enable_pin(pin);
    }
}

void mcu_enable_irq(int irqn)
{
    NVIC_EnableIRQ(irqn);
}

void mcu_disable_irq(int irqn)
{
    NVIC_DisableIRQ(irqn);
}

void mcu_set_irq_prio(int irqn, int preempt, int sub)
{
    int g = NVIC_GetPriorityGrouping();
    NVIC_SetPriority(irqn, NVIC_EncodePriority(g, preempt, sub));
}

void mcu_init_spi(void)
{
    spi_enable_clock(SPI);
    spi_reset(SPI);
    spi_set_master_mode(SPI);
    spi_disable_mode_fault_detect(SPI);
    spi_disable_loopback(SPI);
    spi_set_transfer_delay(SPI, 0, 0, 0);
    spi_set_bits_per_transfer(SPI, 0, SPI_CSR_BITS_8_BIT);
    spi_set_baudrate_div(SPI, 0, 16);
    spi_configure_cs_behavior(SPI, 0, SPI_CS_KEEP_LOW);
    spi_set_clock_polarity(SPI, 0, 0);
    spi_set_clock_phase(SPI, 0, 1);
    spi_enable(SPI);
}

bool mcu_spi_write(uint8_t b)
{
    return spi_write(SPI, b, 0, 0) != SPI_OK;
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

/* configUSE_STATIC_ALLOCATION is set to 1, so the application must provide an
 * implementation of vApplicationGetIdleTaskMemory() to provide the memory that is
 * used by the Idle task. */
void vApplicationGetIdleTaskMemory( StaticTask_t **ppxIdleTaskTCBBuffer,
        StackType_t **ppxIdleTaskStackBuffer,
        uint32_t *pulIdleTaskStackSize )
{
    static StaticTask_t xIdleTaskTCB;
    static StackType_t uxIdleTaskStack[ configMINIMAL_STACK_SIZE ];

    *ppxIdleTaskTCBBuffer = &xIdleTaskTCB;

    *ppxIdleTaskStackBuffer = uxIdleTaskStack;

    *pulIdleTaskStackSize = configMINIMAL_STACK_SIZE;
}
/*-----------------------------------------------------------*/

/* configUSE_STATIC_ALLOCATION and configUSE_TIMERS are both set to 1, so the
 * application must provide an implementation of vApplicationGetTimerTaskMemory()
 * to provide the memory that is used by the Timer service task. */
void vApplicationGetTimerTaskMemory( StaticTask_t **ppxTimerTaskTCBBuffer,
        StackType_t **ppxTimerTaskStackBuffer,
        uint32_t *pulTimerTaskStackSize )
{
    static StaticTask_t xTimerTaskTCB;
    static StackType_t uxTimerTaskStack[ configTIMER_TASK_STACK_DEPTH ];

    *ppxTimerTaskTCBBuffer = &xTimerTaskTCB;

    *ppxTimerTaskStackBuffer = uxTimerTaskStack;

    *pulTimerTaskStackSize = configTIMER_TASK_STACK_DEPTH;
}

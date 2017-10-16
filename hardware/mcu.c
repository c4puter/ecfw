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

void mcu_use_external_clock(bool ext)
{
    struct pll_config pllcfg;

    pmc_switch_mck_to_mainck(1);

    if (ext) {
        pmc_osc_bypass_main_xtal();
        pll_enable_source(CONFIG_PLL1_SOURCE);
        pll_config_defaults(&pllcfg, 1);
        pll_enable(&pllcfg, 1);
        pll_wait_for_lock(1);
        pmc_switch_mck_to_pllbck(CONFIG_SYSCLK_PRES);
        pll_disable(0);
    } else {
        pll_enable_source(CONFIG_PLL0_SOURCE);
        pll_config_defaults(&pllcfg, 0);
        pll_enable(&pllcfg, 0);
        pll_wait_for_lock(0);
        pmc_switch_mck_to_pllack(CONFIG_SYSCLK_PRES);
        pll_disable(1);
    }
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
    spi_set_baudrate_div(SPI, 0, 4);
    spi_configure_cs_behavior(SPI, 0, SPI_CS_KEEP_LOW);
    spi_set_clock_polarity(SPI, 0, 0);
    spi_set_clock_phase(SPI, 0, 1);
    spi_enable(SPI);
}

bool mcu_spi_write(uint8_t b)
{
    return spi_write(SPI, b, 0, 0) != SPI_OK;
}

uint32_t mcu_spi_pdc_base(void)
{
    return spi_get_pdc_base(SPI);
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

#define NB_PIO PIOC
uint32_t const clk_bm   = 1u << 14;
uint32_t const nrd_bm   = 1u << 11;
uint32_t const start_bm = 1u << 8;
uint32_t const nwait_bm = 1u << 13;

/* Similar functions are declared in pio.h, but defined in pio.c, and without
 * LTO we end up with actual function calls. Too slow for this.
 */

#define get_output_write_status(p)      ((p)->PIO_OWSR)
#define enable_output_write(p, mask)    ((p)->PIO_OWER = (mask))
#define disable_output_write(p, mask)   ((p)->PIO_OWDR = (mask))
#define write_pins(p, mask)             ((p)->PIO_ODSR = (mask)) // masked
#define set_pins(p, mask)               ((p)->PIO_SODR = (mask))
#define clear_pins(p, mask)             ((p)->PIO_CODR = (mask))
#define get_pins(p, mask)               ((p)->PIO_PDSR & (mask))
#define pins_output(p, mask)            ((p)->PIO_OER = (mask))
#define pins_input(p, mask)             ((p)->PIO_ODR = (mask))

static void nb_send_addr(uint64_t dest_addr)
{
    uint8_t const addr0 = (dest_addr & 0x00000000FF) >> 0;
    uint8_t const addr1 = (dest_addr & 0x000000FF00) >> 8;
    uint8_t const addr2 = (dest_addr & 0x0000FF0000) >> 16;
    uint8_t const addr3 = (dest_addr & 0x00FF000000) >> 24;
    uint8_t const addr4 = (dest_addr & 0x0F00000000) >> 32;

    set_pins(NB_PIO, nrd_bm | start_bm);
    pins_output(NB_PIO, 0xff);

    while (!get_pins(NB_PIO, nwait_bm));

    __disable_irq();

    uint8_t owsr = get_output_write_status(NB_PIO);
    disable_output_write(NB_PIO, 0xffffffff);
    enable_output_write(NB_PIO, clk_bm | 0xff | start_bm);

    write_pins(NB_PIO, addr0 | start_bm);
    set_pins(NB_PIO, clk_bm);

    write_pins(NB_PIO, addr1);
    set_pins(NB_PIO, clk_bm);

    write_pins(NB_PIO, addr2);
    set_pins(NB_PIO, clk_bm);

    write_pins(NB_PIO, addr3);
    set_pins(NB_PIO, clk_bm);

    write_pins(NB_PIO, addr4);
    set_pins(NB_PIO, clk_bm);

    disable_output_write(NB_PIO, 0xffffffff);
    enable_output_write(NB_PIO, owsr);

    __enable_irq();
}

static void nb_send_data(uint32_t data)
{
    uint8_t const data0 = (data & 0x000000FF) >> 0;
    uint8_t const data1 = (data & 0x0000FF00) >> 8;
    uint8_t const data2 = (data & 0x00FF0000) >> 16;
    uint8_t const data3 = (data & 0xFF000000) >> 24;

    set_pins(NB_PIO, nrd_bm);
    pins_output(NB_PIO, 0xff);

    __disable_irq();

    uint8_t owsr = get_output_write_status(NB_PIO);
    disable_output_write(NB_PIO, 0xffffffff);
    enable_output_write(NB_PIO, clk_bm | 0xff);

    write_pins(NB_PIO, data0);
    set_pins(NB_PIO, clk_bm);

    write_pins(NB_PIO, data1);
    set_pins(NB_PIO, clk_bm);

    write_pins(NB_PIO, data2);
    set_pins(NB_PIO, clk_bm);

    write_pins(NB_PIO, data3);
    set_pins(NB_PIO, clk_bm);

    disable_output_write(NB_PIO, 0xffffffff);
    enable_output_write(NB_PIO, owsr);

    __enable_irq();

    while (!get_pins(NB_PIO, nwait_bm));
}

static uint32_t nb_get_data(void)
{
    pins_input(NB_PIO, 0xff);
    clear_pins(NB_PIO, nrd_bm);

    clear_pins(NB_PIO, clk_bm);
    set_pins(NB_PIO, clk_bm);
    while (!get_pins(NB_PIO, nwait_bm));

    uint32_t const data0 = get_pins(NB_PIO, 0xff);
    clear_pins(NB_PIO, clk_bm);
    set_pins(NB_PIO, clk_bm);

    uint32_t const data1 = get_pins(NB_PIO, 0xff);
    clear_pins(NB_PIO, clk_bm);
    set_pins(NB_PIO, clk_bm);

    uint32_t const data2 = get_pins(NB_PIO, 0xff);
    clear_pins(NB_PIO, clk_bm);
    set_pins(NB_PIO, clk_bm);

    uint32_t const data3 = get_pins(NB_PIO, 0xff);

    return (data0 << 0) | (data1 << 8) | (data2 << 16) | (data3 << 24);
}

static void nb_finish_read(void)
{
    set_pins(NB_PIO, nrd_bm);
    pins_output(NB_PIO, 0xff);
    while (!get_pins(NB_PIO, nwait_bm));
}

/**
 * Write a block of data into the northbridge. Data must comprise 32-bit words
 * and the destination is word-addressed. Length is the number of words to
 * write.
 */
void northbridge_poke(uint64_t dest_addr, uint32_t const *src, uint32_t n)
{
    for (uint32_t i = 0; i < n; ++i) {
        uint64_t const dest_addr_i = dest_addr + i;

        if (i == 0 || (dest_addr_i & 0xFF) == 0) {
            // Northbridge interface has autoincrement over the last octet of
            // the address. If we would overflow this autoincrement, we must
            // retransmit the address.
            nb_send_addr(dest_addr_i);
        }

        nb_send_data(src[i]);
    }
}

/**
 * Read a block of data from the northbridge. Data must comprise 32-bit words
 * and the source is word-addressed. Length is the number of words to read.
 */
void northbridge_peek(uint32_t *dest, uint64_t src_addr, uint32_t n)
{
    for (uint32_t i = 0; i < n; ++i) {
        uint64_t const src_addr_i = src_addr + i;

        if (i == 0 || (src_addr_i & 0xFF) == 0) {
            // Northbridge interface has autoincrement over the last octet of
            // the address. If we would overflow this autoincrement, we must
            // retransmit the address.
            if (i != 0) {
                nb_finish_read();
            }
            nb_send_addr(src_addr_i);
        }

        dest[i] = nb_get_data();
    }

    nb_finish_read();
}

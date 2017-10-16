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

#ifndef MCU_H
#define MCU_H

#include <stdbool.h>
#include <stdint.h>

void mcu_init(void);
void write_stack_canaries(void);
uint32_t get_stack_unused(void);

void mcu_use_external_clock(bool ext);

unsigned int mcu_get_peripheral_hz(void);
bool mcu_get_pin_level(unsigned int pin);
void mcu_set_pin_level(unsigned int pin, bool value);
void mcu_init_pin(unsigned int pin, unsigned int mode_mask, bool default_value);

void mcu_enable_irq(int irqn);
void mcu_disable_irq(int irqn);
void mcu_set_irq_prio(int irqn, int preempt, int sub);

void mcu_init_spi(void);
// return true on timeout
bool mcu_spi_write(uint8_t b);
// return SPI PDC (DMA controller) base address
uint32_t mcu_spi_pdc_base(void);

// Write a block of data into the northbridge. Data must comprise 32-bit words
// and the destination is word-addresed. Length is the number of words to write.
// Not threadsafe - a driver wrapping this should guard itself with a mutex.
void northbridge_poke(uint64_t dest_addr, uint32_t const *src, uint32_t n);

// Read a block of data from the northbridge. Data must comprise 32-bit words
// and the source is word-addressed. Length is the number of words to read.
// Not threadsafe - a driver wrapping this should guard itself with a mutex.
void northbridge_peek(uint32_t *dest, uint64_t source_addr, uint32_t n);

#endif // MCU_H

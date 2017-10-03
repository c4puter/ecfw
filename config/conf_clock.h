#ifndef CONF_CLOCK_H
#define CONF_CLOCK_H

#define BOARD_MCK (120u*1000000u)

#define CONFIG_SYSCLK_SOURCE        SYSCLK_SRC_PLLACK
#define CONFIG_SYSCLK_PRES          SYSCLK_PRES_2
#define CONFIG_PLL0_SOURCE          PLL_SRC_MAINCK_12M_RC
#define CONFIG_PLL0_MUL             20
#define CONFIG_PLL0_DIV             1

// PLL1 is used to derive the same 120 MHz clock from the onboard clock synth
// rather than the internal RC.
//
// Clock synth:
//  20 MHz crystal
//  PLL factor 75/8
//  Output divider 25
//  --- 7.5 MHz ---
//  MCU PLL factor 32
//  --- 240 MHz ---
//  Prescaler: 2
//  --- 120 MHz ---
#define CONFIG_PLL1_SOURCE          PLL_SRC_MAINCK_BYPASS
#define CONFIG_PLL1_MUL             32
#define CONFIG_PLL1_DIV             1

#define BOARD_FREQ_MAINCK_XTAL      (12u*1000000u)
#define BOARD_FREQ_MAINCK_BYPASS    (12u*1000000u)
#define BOARD_FREQ_SLCK_XTAL        32768u
#define BOARD_FREQ_SLCK_BYPASS      32768u
#define BOARD_OSC_STARTUP_US        15625

#if 0
#define CONFIG_SYSCLK_SOURCE        SYSCLK_SRC_MAINCK_12M_RC
#define CONFIG_SYSCLK_PRES          SYSCLK_PRES_1
#define BOARD_FREQ_SLCK_XTAL    (12u*1000000u)
#define BOARD_FREQ_SLCK_BYPASS  (12u*1000000u)
#define BOARD_FREQ_MAINCK_XTAL  (12u*1000000u)
#define BOARD_FREQ_MAINCK_BYPASS (12u*1000000u)
#define BOARD_OSC_STARTUP_US    (15625UL)
#endif


#endif // CONF_CLOCK_H

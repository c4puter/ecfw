#include <stdio.h>
#include <asf/boards/board.h>
#include <asf/services/ioport/ioport.h>

#define LED_GPIO IOPORT_CREATE_PIN(PIOC, 17)

void mcu_init(void)
{
    WDT->WDT_MR = WDT_MR_WDDIS;
    sysclk_init();
    ioport_init();
    ioport_enable_pin(LED_GPIO);
    ioport_set_pin_dir(LED_GPIO, IOPORT_DIR_OUTPUT);
}

void board_init(void)
{
}

void do_toggle_led(void)
{
    ioport_toggle_pin_level(LED_GPIO);
}

#ifndef _CONF_USB_H_
#define _CONF_USB_H_

#include <asf/utils/compiler.h>
#include <asf/services/usb/class/cdc/usb_protocol_cdc.h>

#define UDD_NO_SLEEP_MGR

// mcu.c
void main_sof_action(void);
void main_resume_action(void);
void main_suspend_action(void);
bool callback_cdc_enable(uint8_t);
void callback_cdc_disable(uint8_t);
void callback_cdc_set_coding_ext(uint8_t, usb_cdc_line_coding_t *);
void callback_cdc_set_dtr(uint8_t, bool);
void callback_cdc_rx_notify(uint8_t);

#define  USB_DEVICE_VENDOR_ID             0x1209
#define  USB_DEVICE_PRODUCT_ID            0x4757
#define  USB_DEVICE_MAJOR_VERSION         1
#define  USB_DEVICE_MINOR_VERSION         0
#define  USB_DEVICE_POWER                 20 // mA
#define  USB_DEVICE_ATTR                  (USB_CONFIG_ATTR_SELF_POWERED)
#define  USB_DEVICE_MANUFACTURE_NAME      "WCP52"
#define  USB_DEVICE_PRODUCT_NAME          "GPhA 1"
// #define  USB_DEVICE_SERIAL_NAME           "12...EF"

// USB callbacks
#define  UDC_VBUS_EVENT(b_vbus_high)
#define  UDC_SOF_EVENT()                  main_sof_action()
#define  UDC_SUSPEND_EVENT()              main_suspend_action()
#define  UDC_RESUME_EVENT()               main_resume_action()


// CDC library configuration

#define  UDI_CDC_PORT_NB 1 // Number of ports

// CDC callbacks
#define  UDI_CDC_ENABLE_EXT(port)         callback_cdc_enable(port)
#define  UDI_CDC_DISABLE_EXT(port)        callback_cdc_disable(port)
#define  UDI_CDC_RX_NOTIFY(port)          callback_cdc_rx_notify(port)
#define  UDI_CDC_TX_EMPTY_NOTIFY(port)
#define  UDI_CDC_SET_CODING_EXT(port,cfg) callback_cdc_set_coding_ext(port,cfg)
#define  UDI_CDC_SET_DTR_EXT(port,set)    callback_cdc_set_dtr(port,set)
#define  UDI_CDC_SET_RTS_EXT(port,set)

// CDC settings
#define  UDI_CDC_DEFAULT_RATE             115200
#define  UDI_CDC_DEFAULT_STOPBITS         CDC_STOP_BITS_1
#define  UDI_CDC_DEFAULT_PARITY           CDC_PAR_NONE
#define  UDI_CDC_DEFAULT_DATABITS         8
#define  UDI_CDC_LOW_RATE

#include <asf/services/usb/class/cdc/device/udi_cdc_conf.h>

#endif // _CONF_USB_H_

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

use main::power::*;
use main::pins::*;
use core::sync::atomic::*;

pub static SUPPLY_TABLE: &'static [&'static(Supply + Sync)] = &[
    &BUCK_5VA,
    &BUCK_5VB,
    &BUCK_3VA,
    &BUCK_3VB,
    &INV_N12,
    &LDO_S3,
    &LDO_S0,
    &BUCK_1V5,
    &BUCK_1V2,
    &SW1,
    &SW2,
];

pub static BUCK_5VA: VrmSupply = VrmSupply {
    virt: VirtualSupply {
        name: "BUCK_5VA",
        deps: &[],
        refcount: ATOMIC_USIZE_INIT,
    },
    vrm_id: 1,
    was_up: ATOMIC_BOOL_INIT,
};

pub static BUCK_5VB: VrmSupply = VrmSupply {
    virt: VirtualSupply {
        name: "BUCK_5VB",
        deps: &[],
        refcount: ATOMIC_USIZE_INIT,
    },
    vrm_id: 2,
    was_up: ATOMIC_BOOL_INIT,
};

pub static BUCK_3VA: VrmSupply = VrmSupply {
    virt: VirtualSupply {
        name: "BUCK_3VA",
        deps: &[],
        refcount: ATOMIC_USIZE_INIT,
    },
    vrm_id: 3,
    was_up: ATOMIC_BOOL_INIT,
};

pub static BUCK_3VB: VrmSupply = VrmSupply {
    virt: VirtualSupply {
        name: "BUCK_3VB",
        deps: &[],
        refcount: ATOMIC_USIZE_INIT,
    },
    vrm_id: 4,
    was_up: ATOMIC_BOOL_INIT,   // supplies standby power
};

pub static INV_N12: VrmSupply = VrmSupply {
    virt: VirtualSupply {
        name: "INV_N12",
        deps: &[&BUCK_5VA, &BUCK_5VB],
        refcount: ATOMIC_USIZE_INIT,
    },
    vrm_id: 5,
    was_up: ATOMIC_BOOL_INIT,
};

pub static LDO_S3: GpioSwitchedSupply = GpioSwitchedSupply {
    virt: VirtualSupply {
        name: "LDO_S3",
        deps: &[&BUCK_1V5],
        refcount: ATOMIC_USIZE_INIT,
    },
    gpio: &EN_V75REF,
    wait_ticks: 1,
};

pub static LDO_S0: GpioSwitchedSupply = GpioSwitchedSupply {
    virt: VirtualSupply {
        name: "LDO_S0",
        deps: &[&LDO_S3],
        refcount: ATOMIC_USIZE_INIT,
    },
    gpio: &EN_V75,
    wait_ticks: 1,
};

pub static BUCK_1V5: GpioSwitchedSupply = GpioSwitchedSupply {
    virt: VirtualSupply {
        name: "BUCK_1V5",
        deps: &[&BUCK_5VB],
        refcount: ATOMIC_USIZE_INIT,
    },
    gpio: &EN_1V5,
    wait_ticks: 10,
};

pub static BUCK_1V2: GpioSwitchedSupply = GpioSwitchedSupply {
    virt: VirtualSupply {
        name: "BUCK_1V2",
        deps: &[&BUCK_5VA],
        refcount: ATOMIC_USIZE_INIT,
    },
    gpio: &EN_1V2,
    wait_ticks: 10,
};

pub static SW1: GpioSwitchedSupply = GpioSwitchedSupply {
    virt: VirtualSupply {
        name: "SW1",
        deps: &[],
        refcount: ATOMIC_USIZE_INIT,
    },
    gpio: &EN_P12V_PCI,
    wait_ticks: 1,
};

pub static SW2: GpioSwitchedSupply = GpioSwitchedSupply {
    virt: VirtualSupply {
        name: "SW2",
        deps: &[&BUCK_5VB],
        refcount: ATOMIC_USIZE_INIT,
    },
    gpio: &EN_P12V_PCI,
    wait_ticks: 1,
};

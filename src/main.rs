#![no_main]
#![no_std]

pub mod bool_future;
use bool_future::*;

use aux14::i2c1;
use aux14::{entry, iprintln, prelude::*};
use futures::stream;
use futures::stream::StreamExt;

// Slave address
const MAGNETOMETER: u8 = 0b001_1110;

// Addresses of the magnetometer's register that has the magnetic data
const OUT_X_H_M: u8 = 0x03;

async fn get_compass(i2c1: &'static i2c1::RegisterBlock) -> (i16, i16, i16) {
    // Stage 1: Send the address of the register we want to read to the
    // magnetometer
    {
        // TODO Broadcast START
        i2c1.cr2.write(|w| {
            w.start().set_bit();
            w.sadd1().bits(MAGNETOMETER);
            w.rd_wrn().clear_bit();
            w.nbytes().bits(1);
            w.autoend().clear_bit()
        });

        BoolFuture(|| i2c1.isr.read().txis().bit_is_clear()).await;

        i2c1.txdr.write(|w| w.txdata().bits(OUT_X_H_M));

        BoolFuture(|| i2c1.isr.read().tc().bit_is_clear()).await;
    }

    // Stage 2: Receive the contents of the register we asked for
    // TODO Broadcast RESTART
    // TODO Broadcast the MAGNETOMETER address with the R/W bit set to Read
    i2c1.cr2.modify(|_, w| {
        w.start().set_bit();
        w.nbytes().bits(6);
        w.rd_wrn().set_bit();
        w.autoend().set_bit()
    });

    BoolFuture(|| i2c1.isr.read().rxne().bit_is_clear()).await;

    let mut buffer = [0u8; 6];

    for byte in &mut buffer {
        while i2c1.isr.read().rxne().bit_is_clear() {}

        *byte = i2c1.rxdr.read().rxdata().bits();
    }

    let x_h = u16::from(buffer[0]);
    let x_l = u16::from(buffer[1]);
    let z_h = u16::from(buffer[2]);
    let z_l = u16::from(buffer[3]);
    let y_h = u16::from(buffer[4]);
    let y_l = u16::from(buffer[5]);

    let x = ((x_h << 8) + x_l) as i16;
    let y = ((y_h << 8) + y_l) as i16;
    let z = ((z_h << 8) + z_l) as i16;

    (x, y, z)
}

#[entry]
fn main() -> ! {
    let (i2c1, mut delay, mut itm) = aux14::init();

    bool_future::block_on(
        stream::repeat(())
            .then(|()| get_compass(i2c1))
            .map(|mag| {
                iprintln!(&mut itm.stim[0], "{:?}", mag);
                delay.delay_ms(1000_u16);
            })
            .for_each(|()| futures::future::ready(())),
    );

    unreachable!("infinite stream")
}

//! Async driver implementation for Waveshare ESP32-S3 1.8" AMOLED
//! Uses QSPI interface and I2C-based GPIO expander or GPIO for reset.

use crate::{ControllerInterfaceAsync, ResetInterfaceAsync};
use esp_hal::{
    spi::{
        master::{Address, Command, DataMode, SpiDmaBus},
        Error as SpiError,
    },
    Async,
};

const CMD_RAMWR: u32 = 0x2C;
const CMD_RAMWRC: u32 = 0x3C;
const QSPI_PIXEL_OPCODE: u8 = 0x32;
const QSPI_CONTROL_OPCODE: u8 = 0x02;
pub const DMA_CHUNK_SIZE_ASYNC: usize = 16380;

/// QSPI implementation of ControllerInterface for SH8601
pub struct Ws18AmoledDriverAsync {
    pub qspi: SpiDmaBus<'static, Async>,
}

impl Ws18AmoledDriverAsync {
    pub fn new(qspi: SpiDmaBus<'static, Async>) -> Self {
        Ws18AmoledDriverAsync { qspi }
    }
}

impl ControllerInterfaceAsync for Ws18AmoledDriverAsync {
    type Error = SpiError;

    async fn send_command(&mut self, cmd: u8) -> Result<(), Self::Error> {
        self.send_command_with_data(cmd, &[]).await
    }

    async fn send_command_with_data(&mut self, cmd: u8, data: &[u8]) -> Result<(), Self::Error> {
        let address_value = (cmd as u32) << 8;

        self.qspi
            .half_duplex_write(
                DataMode::Single,
                Command::_8Bit(QSPI_CONTROL_OPCODE as u16, DataMode::Single),
                Address::_24Bit(address_value, DataMode::Single),
                0,
                data,
            )?;
        Ok(())
    }

    async fn send_pixels(&mut self, pixels: &[u8]) -> Result<(), Self::Error> {
        let ramwr_addr_val = (CMD_RAMWR as u32) << 8;
        let ramwrc_addr_val = (CMD_RAMWRC as u32) << 8;

        let mut chunks = pixels.chunks(DMA_CHUNK_SIZE_ASYNC);

        // Send the first chunk with CMD_RAMWR
        if let Some(first_chunk) = chunks.next() {
            self.qspi
                .half_duplex_write(
                    DataMode::Quad,
                    Command::_8Bit(QSPI_PIXEL_OPCODE as u16, DataMode::Single),
                    Address::_24Bit(ramwr_addr_val, DataMode::Single),
                    0,
                    first_chunk,
                )?;
        } else {
            // No pixels to send, so we're done.
            return Ok(());
        }

        // Send all subsequent chunks with CMD_RAMWRC
        for chunk in chunks {
            self.qspi
                .half_duplex_write(
                    DataMode::Quad,
                    Command::_8Bit(QSPI_PIXEL_OPCODE as u16, DataMode::Single),
                    Address::_24Bit(ramwrc_addr_val, DataMode::Single),
                    0,
                    chunk,
                )?;
        }

        Ok(())
    }
}

/// I2C-controlled GPIO Reset Pin
pub struct ResetDriverAsync<I2C> {
    i2c: I2C,
}

impl<I2C> ResetDriverAsync<I2C> {
    pub fn new(i2c: I2C) -> Self {
        ResetDriverAsync { i2c }
    }
}

impl<I2C> ResetInterfaceAsync for ResetDriverAsync<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    type Error = I2C::Error;

    async fn reset(&mut self) -> Result<(), Self::Error> {
        self.i2c.write(0x20, &[0x03, 0x00]).await?; // Configure as output
        self.i2c.write(0x20, &[0x01, 0b0000_0010]).await?; // Drive low
        embassy_time::Timer::after_millis(20).await;
        self.i2c.write(0x20, &[0x01, 0b0000_0111]).await?; // Drive high
        embassy_time::Timer::after_millis(150).await;
        Ok(())
    }
}

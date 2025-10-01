#![no_std]
#![no_main]

use sh8601_rs::{
    framebuffer_size, ColorMode, DisplaySize, ResetDriverAsync, Sh8601DriverAsync,
    Ws18AmoledDriverAsync, DMA_CHUNK_SIZE_ASYNC,
};

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, LineHeight, Text, TextStyleBuilder},
};

extern crate alloc;
use esp_alloc as _;
use esp_backtrace as _;
use esp_bootloader_esp_idf::esp_app_desc;
use esp_hal::{
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers,
    i2c::master::{Config as I2cConfig, I2c},
    spi::{master::{Config as SpiConfig, Spi}, Mode},
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;
use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};

esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // --- DMA Buffers for SPI ---
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(DMA_CHUNK_SIZE_ASYNC);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    // SPI Configuration for Waveshare ESP32-S3 1.8inch AMOLED Touch Display
    // Hardware is configured for QSPI. Pinout obtained from the schematic.
    let lcd_spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(40_u32))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sio0(peripherals.GPIO4)
    .with_sio1(peripherals.GPIO5)
    .with_sio2(peripherals.GPIO6)
    .with_sio3(peripherals.GPIO7)
    .with_cs(peripherals.GPIO12)
    .with_sck(peripherals.GPIO11)
    .with_dma(peripherals.DMA_CH0)
    .with_buffers(dma_rx_buf, dma_tx_buf)
    .into_async();

    // I2C Configuration for Waveshare ESP32-S3 1.8inch AMOLED Touch Display
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO15)
    .with_scl(peripherals.GPIO14)
    .into_async();

    // Initialize I2C GPIO Reset Pin for the WaveShare 1.8" AMOLED display
    let reset = ResetDriverAsync::new(i2c);

    // Initialize display driver for the Waveshare 1.8" AMOLED display
    let ws_driver = Ws18AmoledDriverAsync::new(lcd_spi);

    // Set up the display size
    const DISPLAY_SIZE: DisplaySize = DisplaySize::new(368, 448);

    // Calculate framebuffer size based on the display size and color mode
    const FB_SIZE: usize = framebuffer_size(DISPLAY_SIZE, ColorMode::Rgb565);

    // Instantiate and Initialize Display
    println!("Initializing SH8601 Display (Async)...");
    let display_res = Sh8601DriverAsync::new_heap::<_, FB_SIZE>(
        ws_driver,
        reset,
        ColorMode::Rgb565,
        DISPLAY_SIZE,
        Delay,
    )
    .await;
    
    let mut display = match display_res {
        Ok(d) => {
            println!("Display initialized successfully.");
            d
        }
        Err(e) => {
            println!("Error initializing display: {:?}", e);
            loop {
                Timer::after_millis(1000).await;
            }
        }
    };

    let character_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);

    let text_style = TextStyleBuilder::new()
        .line_height(LineHeight::Pixels(300))
        .alignment(Alignment::Center)
        .build();

    let text = "Hello, Waveshare 1.8\" AMOLED Display (Async)!";

    // Animation loop
    for col in (0..DISPLAY_SIZE.width as i32).step_by(10) {
        Text::with_text_style(text, Point::new(col, 100), character_style, text_style)
            .draw(&mut display)
            .unwrap();
        
        if let Err(e) = display.flush().await {
            println!("Error flushing display: {:?}", e);
        }
        
        Timer::after_millis(500).await;
        display.clear(Rgb565::BLACK).unwrap();
    }

    loop {
        Timer::after_millis(500).await;
    }
}

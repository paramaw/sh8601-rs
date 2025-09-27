use crate::{ControllerInterface, DrawTarget, ResetInterface, Sh8601Driver};
use embedded_graphics_core::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};

impl<IFACE, RST> DrawTarget for Sh8601Driver<IFACE, RST>
where
    IFACE: ControllerInterface,
    RST: ResetInterface,
{
    type Color = Rgb888;
    // Drawing to the framebuffer in memory is infallible.
    // Errors happen during flush with SPI comms.
    type Error = core::convert::Infallible;

    /// Draws a single pixel to the internal framebuffer.
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= 0
                && coord.x < self.config.width as i32
                && coord.y >= 0
                && coord.y < self.config.height as i32
            {
                let x = coord.x as u32;
                let y = coord.y as u32;
                let index = ((y * self.config.width as u32 + x) * 3) as usize;

                if index + 2 < self.framebuffer.len() {
                    let r = (color.into_storage() >> 16) as u8; // 8-bit Red
                    let g = (color.into_storage() >> 8) as u8; // 8-bit Green
                    let b = color.into_storage() as u8; // 8-bit Blue

                    self.framebuffer[index] = r as u8;
                    self.framebuffer[index + 1] = g as u8;
                    self.framebuffer[index + 2] = b as u8;
                }
            }
        }
        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let drawable_area = area.intersection(&self.bounding_box());

        if !drawable_area.is_zero_sized() {
            let mut colors = colors.into_iter();
            // bottom_right is inclusive, so we use ..= to include it in the range.
            // unwrap is safe because we've already checked that the area is not zero-sized.
            let bottom_right = drawable_area.bottom_right().unwrap();

            for y in drawable_area.top_left.y..=bottom_right.y {
                let start_x = drawable_area.top_left.x as u32;
                let end_x = bottom_right.x as u32;
                let width = self.config.width as u32;

                let start_index = ((y as u32 * width + start_x) * 3) as usize;
                let end_index = ((y as u32 * width + end_x) * 3) as usize;

                if let Some(row_buffer) = self.framebuffer.get_mut(start_index..end_index) {
                    for (chunk, color) in row_buffer.chunks_exact_mut(3).zip(&mut colors) {
                        let color_bits = color.into_storage();
                        chunk[0] = (color_bits >> 16) as u8;
                        chunk[1] = (color_bits >> 8) as u8;
                        chunk[2] = color_bits as u8;
                    }
                }
            }
        }

        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let drawable_area = area.intersection(&self.bounding_box());

        if !drawable_area.is_zero_sized() {
            let r = (color.into_storage() >> 16) as u8;
            let g = (color.into_storage() >> 8) as u8;
            let b = color.into_storage() as u8;

            let bottom_right = drawable_area.bottom_right().unwrap();

            for y in drawable_area.top_left.y..=bottom_right.y {
                let start_x = drawable_area.top_left.x as u32;
                let end_x = bottom_right.x as u32;
                let width = self.config.width as u32;

                let start_index = ((y as u32 * width + start_x) * 3) as usize;
                let end_index = ((y as u32 * width + end_x) * 3) as usize;

                if let Some(row_buffer) = self.framebuffer.get_mut(start_index..end_index) {
                    for chunk in row_buffer.chunks_exact_mut(3) {
                        chunk[0] = r;
                        chunk[1] = g;
                        chunk[2] = b;
                    }
                }
            }
        }

        Ok(())
    }
}

// =========== embedded-graphics OriginDimensions Implementation ===========

impl<IFACE, RST> OriginDimensions for Sh8601Driver<IFACE, RST>
where
    IFACE: ControllerInterface,
    RST: ResetInterface,
{
    fn size(&self) -> Size {
        Size::new((self.config.width) as u32, (self.config.height) as u32)
    }
}

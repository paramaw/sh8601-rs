use crate::{ControllerInterfaceAsync, DrawTarget, ResetInterfaceAsync, Sh8601DriverAsync};
use embedded_graphics_core::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};

/// External framebuffer wrapper for double buffering.
/// Allows drawing to a user-managed buffer instead of the driver's internal buffer.
pub struct ExternalFramebuffer<'a> {
    pub buffer: &'a mut [u8],
    pub width: u16,
    pub height: u16,
}

impl<'a> ExternalFramebuffer<'a> {
    /// Creates a new external framebuffer wrapper.
    /// Buffer must be sized for width * height * 3 bytes (RGB888).
    pub fn new(buffer: &'a mut [u8], width: u16, height: u16) -> Self {
        Self { buffer, width, height }
    }
}

impl<'a> DrawTarget for ExternalFramebuffer<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= 0
                && coord.x < self.width as i32
                && coord.y >= 0
                && coord.y < self.height as i32
            {
                let x = coord.x as u32;
                let y = coord.y as u32;
                let index = ((y * self.width as u32 + x) * 3) as usize;

                if index + 2 < self.buffer.len() {
                    let r = (color.into_storage() >> 16) as u8;
                    let g = (color.into_storage() >> 8) as u8;
                    let b = color.into_storage() as u8;

                    self.buffer[index] = r;
                    self.buffer[index + 1] = g;
                    self.buffer[index + 2] = b;
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
            let bottom_right = drawable_area.bottom_right().unwrap();

            for y in drawable_area.top_left.y..=bottom_right.y {
                let start_x = drawable_area.top_left.x as u32;
                let end_x = bottom_right.x as u32;
                let width = self.width as u32;

                let start_index = ((y as u32 * width + start_x) * 3) as usize;
                let end_index = ((y as u32 * width + end_x) * 3) as usize;

                if let Some(row_buffer) = self.buffer.get_mut(start_index..end_index) {
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
                let width = self.width as u32;

                let start_index = ((y as u32 * width + start_x) * 3) as usize;
                let end_index = ((y as u32 * width + end_x) * 3) as usize;

                if let Some(row_buffer) = self.buffer.get_mut(start_index..end_index) {
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

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let r = (color.into_storage() >> 16) as u8;
        let g = (color.into_storage() >> 8) as u8;
        let b = color.into_storage() as u8;

        for chunk in self.buffer.chunks_exact_mut(3) {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
        }

        Ok(())
    }
}

impl<'a> OriginDimensions for ExternalFramebuffer<'a> {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

impl<IFACE, RST> DrawTarget for Sh8601DriverAsync<IFACE, RST>
where
    IFACE: ControllerInterfaceAsync,
    RST: ResetInterfaceAsync,
{
    type Color = Rgb888;
    type Error = core::convert::Infallible;

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
                    let r = (color.into_storage() >> 16) as u8;
                    let g = (color.into_storage() >> 8) as u8;
                    let b = color.into_storage() as u8;

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

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let r = (color.into_storage() >> 16) as u8;
        let g = (color.into_storage() >> 8) as u8;
        let b = color.into_storage() as u8;

        for chunk in self.framebuffer.chunks_exact_mut(3) {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
        }

        Ok(())
    }
}

impl<IFACE, RST> OriginDimensions for Sh8601DriverAsync<IFACE, RST>
where
    IFACE: ControllerInterfaceAsync,
    RST: ResetInterfaceAsync,
{
    fn size(&self) -> Size {
        Size::new((self.config.width) as u32, (self.config.height) as u32)
    }
}

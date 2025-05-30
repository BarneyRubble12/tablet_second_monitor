use crate::config::CaptureConfig;
use crate::error::AppError;
use core_graphics::display::CGDisplay;
use image::{ImageBuffer, Rgba};
use std::time::Duration;
use std::thread;

pub struct ScreenCapture {
    config: CaptureConfig,
    displays: Vec<CGDisplay>,
    current_display: usize,
}

impl ScreenCapture {
    pub fn new(config: CaptureConfig) -> Result<Self, AppError> {
        // Try to get displays with retries
        let mut retries = 3;
        let mut displays = Vec::new();
        
        while retries > 0 {
            match CGDisplay::active_displays() {
                Ok(active_displays) => {
                    displays = active_displays.into_iter()
                        .map(|id| CGDisplay::new(id))
                        .collect();
                    if !displays.is_empty() {
                        break;
                    }
                }
                Err(_) => {}
            }
            retries -= 1;
            if retries > 0 {
                thread::sleep(Duration::from_secs(1));
            }
        }

        if displays.is_empty() {
            return Err(AppError::CaptureError(
                "No displays found. Please make sure screen recording permission is granted in System Settings > Privacy & Security > Screen Recording and restart the application.".to_string()
            ));
        }

        // Verify we can actually capture
        if let Some(display) = displays.first() {
            if display.image().is_none() {
                return Err(AppError::CaptureError(
                    "Screen recording permission not granted. Please grant permission in System Settings > Privacy & Security > Screen Recording and restart the application.".to_string()
                ));
            }
        }

        Ok(Self {
            config,
            displays,
            current_display: 0,
        })
    }

    pub fn get_display_count(&self) -> usize {
        self.displays.len()
    }

    pub fn set_display(&mut self, index: usize) -> Result<(), AppError> {
        if index >= self.displays.len() {
            return Err(AppError::CaptureError(format!(
                "Invalid display index: {}",
                index
            )));
        }
        self.current_display = index;
        Ok(())
    }

    pub fn capture_frame(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, AppError> {
        let display = &self.displays[self.current_display];
        let bounds = display.bounds();
        
        // Create a bitmap context
        let width = bounds.size.width as u32;
        let height = bounds.size.height as u32;
        let bytes_per_row = (width as usize) * 4;
        let mut data = vec![0u8; bytes_per_row * height as usize];

        let mut context = core_graphics::context::CGContext::create_bitmap_context(
            None,
            width as usize,
            height as usize,
            8,
            bytes_per_row as usize,
            &core_graphics::color_space::CGColorSpace::create_device_rgb(),
            core_graphics::base::kCGImageAlphaPremultipliedLast,
        );

        // Draw the display into the context
        let image = display.image().ok_or_else(|| {
            AppError::CaptureError(
                "Failed to capture screen. Please check screen recording permissions and restart the application.".to_string()
            )
        })?;

        context.draw_image(bounds, &image);

        // Get the image data
        let width = context.width();
        let height = context.height();
        let data_ptr = context.data();
        let data_len = width * height * 4;
        data.copy_from_slice(&data_ptr[..data_len]);

        // Convert to ImageBuffer
        let width_u32 = width as u32;
        let height_u32 = height as u32;
        let mut buffer = ImageBuffer::new(width_u32, height_u32);
        for y in 0..height_u32 {
            for x in 0..width_u32 {
                let offset = (y as usize * bytes_per_row + x as usize * 4) as usize;
                if offset + 3 < data.len() {
                    let r = data[offset];
                    let g = data[offset + 1];
                    let b = data[offset + 2];
                    let a = data[offset + 3];
                    buffer.put_pixel(x, y, Rgba([r, g, b, a]));
                }
            }
        }

        Ok(buffer)
    }

    pub fn frame_duration(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.config.fps as f64)
    }
} 
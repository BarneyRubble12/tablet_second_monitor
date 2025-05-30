use crate::config::CaptureConfig;
use crate::error::AppError;
use core_graphics::display::CGDisplay;
use image::{ImageBuffer, Rgba};
use std::time::Duration;

pub struct ScreenCapture {
    config: CaptureConfig,
    displays: Vec<CGDisplay>,
    current_display: usize,
}

impl ScreenCapture {
    pub fn new(config: CaptureConfig) -> Result<Self, AppError> {
        // Check if we have screen recording permission
        if !Self::has_screen_recording_permission() {
            return Err(AppError::CaptureError(
                "Screen recording permission not granted. Please grant permission in System Settings > Privacy & Security > Screen Recording".to_string()
            ));
        }

        let displays: Vec<_> = CGDisplay::active_displays()
            .map_err(|e| AppError::CaptureError(format!("Failed to get displays: {}", e)))?
            .into_iter()
            .map(|id| CGDisplay::new(id))
            .collect();
        
        if displays.is_empty() {
            return Err(AppError::CaptureError("No displays found".to_string()));
        }

        Ok(Self {
            config,
            displays,
            current_display: 0,
        })
    }

    fn has_screen_recording_permission() -> bool {
        // Try to capture a small portion of the screen to check permissions
        if let Ok(displays) = CGDisplay::active_displays() {
            if let Some(id) = displays.first() {
                let display = CGDisplay::new(*id);
                if let Some(_) = display.image() {
                    return true;
                }
            }
        }
        false
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
            if !Self::has_screen_recording_permission() {
                AppError::CaptureError(
                    "Screen recording permission not granted. Please grant permission in System Settings > Privacy & Security > Screen Recording".to_string()
                )
            } else {
                AppError::CaptureError("Failed to get display image".to_string())
            }
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
use crate::capture::ScreenCapture;
use crate::error::AppError;
use image::{ImageBuffer, Rgba};
use std::time::Duration;
use tokio::time;

pub struct ScreenStream {
    capture: ScreenCapture,
    last_frame: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    quality: u8,
    frame_count: u32,
}

impl ScreenStream {
    pub fn new(capture: ScreenCapture) -> Self {
        Self {
            capture,
            last_frame: None,
            quality: 80,
            frame_count: 0,
        }
    }

    pub fn set_quality(&mut self, quality: u8) {
        self.quality = quality.clamp(1, 100);
    }

    fn calculate_frame_diff(
        &self,
        current: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        last: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    ) -> f32 {
        if current.dimensions() != last.dimensions() {
            return 1.0;
        }

        let mut diff_pixels = 0;
        let total_pixels = current.width() * current.height();

        for y in 0..current.height() {
            for x in 0..current.width() {
                let current_pixel = current.get_pixel(x, y);
                let last_pixel = last.get_pixel(x, y);
                if current_pixel != last_pixel {
                    diff_pixels += 1;
                }
            }
        }

        diff_pixels as f32 / total_pixels as f32
    }

    pub async fn start_streaming<F>(&mut self, mut callback: F) -> Result<(), AppError>
    where
        F: FnMut(&[u8]) -> Result<(), AppError>,
    {
        let mut interval = time::interval(self.capture.frame_duration());

        loop {
            interval.tick().await;
            
            let frame = self.capture.capture_frame()?;
            self.frame_count += 1;

            // Check if we should send this frame
            let should_send = if let Some(ref last_frame) = self.last_frame {
                let diff = self.calculate_frame_diff(&frame, last_frame);
                diff > 0.01 // Send if more than 1% of pixels changed
            } else {
                true // Always send the first frame
            };

            if should_send {
                // Convert to JPEG with quality control
                let mut jpeg_data = Vec::new();
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    std::io::Cursor::new(&mut jpeg_data),
                    self.quality,
                );
                
                encoder.encode(
                    frame.as_raw(),
                    frame.width(),
                    frame.height(),
                    image::ColorType::Rgba8,
                ).map_err(|e| AppError::CaptureError(format!("Failed to encode JPEG: {}", e)))?;

                // Send the frame
                callback(&jpeg_data)?;
                
                self.last_frame = Some(frame);
            }
        }
    }
} 
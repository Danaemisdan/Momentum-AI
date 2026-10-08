import re
with open('src/vision_stream.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace shared_memory with a simple file writer
new_content = """use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use xcap::Monitor;
use image::ImageFormat;

pub struct VisionStream {
    pub width: Arc<Mutex<u32>>,
    pub height: Arc<Mutex<u32>>,
}

impl VisionStream {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            width: Arc::new(Mutex::new(0)),
            height: Arc::new(Mutex::new(0)),
        })
    }

    pub fn start(&self) {
        let width_clone = Arc::clone(&self.width);
        let height_clone = Arc::clone(&self.height);
        
        thread::spawn(move || {
            loop {
                // Get primary monitor
                let monitors = Monitor::all().unwrap_or_else(|_| vec![]);
                let primary_monitor = monitors.into_iter().find(|m| m.is_primary().unwrap_or(false));
                
                if let Some(monitor) = primary_monitor.or_else(|| Monitor::all().unwrap_or_else(|_| vec![]).into_iter().next()) {
                    if let Ok(image) = monitor.capture_image() {
                        let w = image.width();
                        let h = image.height();
                        
                        *width_clone.lock().unwrap() = w;
                        *height_clone.lock().unwrap() = h;
                        
                        let temp_file = std::env::temp_dir().join("momentum_vision_current.jpg");
                        // Save directly as JPEG to temp dir
                        let _ = image.save_with_format(temp_file, ImageFormat::Jpeg);
                    }
                }
                
                // Sleep for ~500ms (2 FPS is plenty for agentic vision and saves CPU)
                thread::sleep(Duration::from_millis(500));
            }
        });
    }
}
"""

with open('src/vision_stream.rs', 'w', encoding='utf-8') as f:
    f.write(new_content)

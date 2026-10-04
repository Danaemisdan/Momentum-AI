use shared_memory::{Shmem, ShmemConf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use xcap::Monitor;

pub struct VisionStream {
    shm: Shmem,
    pub width: Arc<Mutex<u32>>,
    pub height: Arc<Mutex<u32>>,
}

impl VisionStream {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Allocate a 64MB shared memory block (enough for 5K RGBA)
        let temp_file = std::env::temp_dir().join("momentum_vision.bin");
        let temp_path = temp_file.to_str().unwrap();
        let shm = match ShmemConf::new()
            .size(64_000_000)
            .force_create_flink()
            .flink(temp_path)
            .create() {
                Ok(m) => {
                    println!("Created Memory Mapped File: {} (64MB)", temp_path);
                    m
                },
                Err(e) => {
                    println!("Failed to create with force ({}), trying to open existing", e);
                    ShmemConf::new().flink(temp_path).open()?
                },
            };

        Ok(Self {
            shm,
            width: Arc::new(Mutex::new(0)),
            height: Arc::new(Mutex::new(0)),
        })
    }

    pub fn start(&self) {
        let width_clone = Arc::clone(&self.width);
        let height_clone = Arc::clone(&self.height);
        
        // We need the raw pointer to the shared memory to write to it safely across threads
        // The memory block is owned by this process and will live as long as the agent lives.
        let shm_ptr = self.shm.as_ptr() as usize;

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
                        
                        let bytes = image.as_raw();
                        let expected_size = (w * h * 4) as usize;
                        
                        // Safety: We allocated 64MB. Ensure we don't overflow.
                        if expected_size + 8 <= 64_000_000 {
                            unsafe {
                                let dst = shm_ptr as *mut u8;
                                // Write width and height as native-endian u32
                                let w_u32 = w as u32;
                                let h_u32 = h as u32;
                                std::ptr::copy_nonoverlapping(&w_u32 as *const u32 as *const u8, dst, 4);
                                std::ptr::copy_nonoverlapping(&h_u32 as *const u32 as *const u8, dst.add(4), 4);
                                // Write raw pixels
                                std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst.add(8), expected_size);
                            }
                        }
                    }
                }
                
                // Sleep for ~500ms (2 FPS is plenty for agentic vision and saves CPU)
                thread::sleep(Duration::from_millis(500));
            }
        });
    }
}

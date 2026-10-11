use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::time::{Instant, Duration};
use tauri::{Manager, Emitter};

pub fn start_notch_hover_listener(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut was_hovered = false;
        loop {
            std::thread::sleep(Duration::from_millis(40));
            
            #[cfg(target_os = "macos")]
            let (x, y) = {
                use core_graphics::event::CGEvent;
                use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
                if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
                    if let Ok(event) = CGEvent::new(source) {
                        let loc = event.location();
                        (loc.x, loc.y)
                    } else {
                        (0.0, 1000.0)
                    }
                } else {
                    (0.0, 1000.0)
                }
            };
            #[cfg(not(target_os = "macos"))]
            let (x, y) = (0.0, 1000.0);

            if let Some(window) = app_handle.get_webview_window("main") {
                if let Ok(Some(monitor)) = window.current_monitor() {
                    let screen_w = monitor.size().width as f64 / monitor.scale_factor();
                    let notch_center = screen_w / 2.0;
                    let notch_left = notch_center - 160.0;
                    let notch_right = notch_center + 160.0;

                    let in_notch_zone = y <= 35.0 && x >= notch_left && x <= notch_right;
                    let in_expanded_zone = y <= 160.0 && x >= (notch_left - 100.0) && x <= (notch_right + 100.0);

                    let is_hovered = if was_hovered { in_expanded_zone } else { in_notch_zone };

                    if is_hovered != was_hovered {
                        was_hovered = is_hovered;
                        let _ = window.emit("notch-hover", is_hovered);
                        if is_hovered {
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        }
    });
}

pub fn start_knock_listener(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = match host.default_input_device() {
            Some(d) => d,
            None => {
                println!("No input device found for knock sensor.");
                return;
            }
        };

        let config: cpal::StreamConfig = match device.default_input_config() {
            Ok(c) => c.into(),
            Err(_) => return,
        };

        let mut last_knock = Instant::now() - Duration::from_secs(10);
        let mut knock_count = 0;

        let err_fn = |err| eprintln!("an error occurred on the input audio stream: {}", err);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut max_amp = 0.0_f32;
                for &sample in data {
                    if sample.abs() > max_amp {
                        max_amp = sample.abs();
                    }
                }

                if max_amp > 0.04 { 
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_knock);
                    
                    // 100ms debounce
                    if elapsed > Duration::from_millis(100) {
                        // Forgiving window for the second tap
                        if elapsed < Duration::from_millis(600) {
                            knock_count += 1;
                        } else {
                            knock_count = 1;
                        }
                        
                        last_knock = now;
                        
                        if knock_count >= 2 {
                            knock_count = 0;
                            println!("💥 Double knock detected! Summoning Momentum...");
                            if let Some(window) = app_handle.get_webview_window("main") {
                                // Must show it first if it's completely hidden so the animation can play
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("visibility-toggle", "knock");
                            }
                        }
                    }
                }
            },
            err_fn,
            None
        );

        if let Ok(stream) = stream {
            if stream.play().is_ok() {
                println!("🎧 Knock sensor active (listening on default mic)...");
                // Keep thread alive forever
                loop {
                    std::thread::sleep(Duration::from_secs(60));
                }
            }
        }
    });
}

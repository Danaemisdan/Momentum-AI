mod types;
mod browser;
mod perception;
mod brain;
mod executor;
mod server;
mod skills;
mod scraper;
mod vision_stream;
pub mod agent;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      
      // Spawn the Momentum Agent Backend
      tauri::async_runtime::spawn(async move {
          if let Err(e) = agent::start_agent().await {
              eprintln!("Momentum Agent crashed: {}", e);
          }
      });
      
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

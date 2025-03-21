use solana_geyser_plugin_manager::geyser_plugin_manager::GeyserPluginManager;
use std::path::Path;

fn main() {
    env_logger::init();

    let mut plugin_manager = GeyserPluginManager::default();

    let plugin_path = Path::new("/home/ubuntu/.config/solana/validator.yml");

    match plugin_manager.load(plugin_path) {
        Ok(_) => {
            println!("✅ Geyser plugin loaded successfully from config.");
        }
        Err(e) => {
            eprintln!("❌ Failed to load plugin: {:?}", e);
        }
    }
}
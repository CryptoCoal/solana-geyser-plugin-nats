use {
    log::*,
    nats::{Connection, Options},
    serde::Serialize,
    solana_geyser_plugin_interface::{
        geyser_plugin_interface::{
            GeyserPlugin, ReplicaAccountInfoVersions, SlotStatus
        },
    },
    std::fs,
};
use solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPluginError;

#[derive(Serialize)]
struct SlotPayload {
    slot: u64,
    parent: Option<u64>,
    status: String,
}

#[derive(Debug)]
struct GeyserNatsPlugin {
    nats_conn: Option<Connection>,
    subject: String,
    cluster: String,
    flush_interval_ms: u64,
}

impl GeyserNatsPlugin {
    fn publish<T: Serialize>(&self, payload: &T) {
        if let Some(conn) = &self.nats_conn {
            if let Ok(json) = serde_json::to_string(payload) {
                let _ = conn.publish(&self.subject, json);
            }
        }
    }
}

impl GeyserPlugin for GeyserNatsPlugin {
    fn name(&self) -> &'static str {
        "GeyserNatsPlugin"
    }

    fn on_load(&mut self, config_file: &str) -> Result<(), GeyserPluginError> {

        println!("[PLUGIN] on_load() CONFIG FILE = {}", config_file);

        let config_str = fs::read_to_string(config_file)?;
        let config: serde_json::Value = serde_json::from_str(&config_str)
             .map_err(|e| GeyserPluginError::Custom(Box::new(e)))?;

        let nats_url = config["nats_url"].as_str().unwrap_or("127.0.0.1");
        self.subject = config["nats_subject"].as_str().unwrap_or("sniper.blocks").to_string();
        self.cluster = config["nats_cluster"].as_str().unwrap_or("solana-sniper").to_string();
        self.flush_interval_ms = config["flush_interval_ms"].as_u64().unwrap_or(50);

        let conn = Options::new().with_name(&self.cluster).connect(nats_url)?;
        self.nats_conn = Some(conn);
        info!("Geyser NATS plugin loaded — streaming to {}", self.subject);
        Ok(())
    }

    fn update_slot_status(
        &self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus,
    ) -> Result<(), GeyserPluginError> {
        let payload = SlotPayload {
            slot,
            parent,
            status: format!("{:?}", status),
        };
        self.publish(&payload);
        Ok(())
    }

    fn update_account(&self, _account: ReplicaAccountInfoVersions, _slot: u64, _is_startup: bool) -> Result<(), GeyserPluginError> {
        // (Optional: You can implement account streaming here)
        Ok(())
    }

    fn notify_end_of_startup(&self) -> Result<(), GeyserPluginError> {
        info!("Geyser NATS plugin finished startup");
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    println!("[PLUGIN] _create_plugin() loaded from .so ✅");

    let plugin = GeyserNatsPlugin {
        nats_conn: None,
        subject: "sniper.blocks".to_string(),
        cluster: "solana-sniper".to_string(),
        flush_interval_ms: 50,
    };

    let boxed: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(boxed)
}
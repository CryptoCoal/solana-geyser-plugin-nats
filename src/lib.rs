use {
    log::*,
    nats::{Connection, Options},
    serde::Serialize,
    serde_json::json,
    solana_geyser_plugin_interface::{
        geyser_plugin_interface::{GeyserPlugin, GeyserPluginSlotInfo, GeyserPluginResult, ReplicaAccountInfoVersions, SlotStatus},
        GeyserPluginManager,
    },
    std::{collections::HashMap, ffi::CStr, fs, path::Path, sync::Mutex},
};

#[derive(Serialize)]
struct SlotPayload {
    slot: u64,
    parent: Option<u64>,
    status: String,
}

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

    fn on_load(&mut self, config_file: &str) -> GeyserPluginResult<()> {
        let config_str = fs::read_to_string(config_file)?;
        let config: serde_json::Value = serde_json::from_str(&config_str)?;

        let nats_url = config["nats_url"].as_str().unwrap_or("127.0.0.1");
        self.subject = config["nats_subject"].as_str().unwrap_or("sniper.blocks").to_string();
        self.cluster = config["nats_cluster"].as_str().unwrap_or("solana-sniper").to_string();
        self.flush_interval_ms = config["flush_interval_ms"].as_u64().unwrap_or(50);

        let conn = Options::new().with_name(&self.cluster).connect(nats_url)?;
        self.nats_conn = Some(conn);
        info!("Geyser NATS plugin loaded — streaming to {}", self.subject);
        Ok(())
    }

    fn update_slot_status(&self, slot_info: &GeyserPluginSlotInfo) -> GeyserPluginResult<()> {
        let payload = SlotPayload {
            slot: slot_info.slot,
            parent: slot_info.parent,
            status: format!("{:?}", slot_info.status),
        };
        self.publish(&payload);
        Ok(())
    }

    fn update_account(&self, _account: ReplicaAccountInfoVersions, _slot: u64, _is_startup: bool) -> GeyserPluginResult<()> {
        // (Optional: You can implement account streaming here)
        Ok(())
    }

    fn notify_end_of_startup(&self) -> GeyserPluginResult<()> {
        info!("Geyser NATS plugin finished startup");
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = GeyserNatsPlugin {
        nats_conn: None,
        subject: "sniper.blocks".to_string(),
        cluster: "solana-sniper".to_string(),
        flush_interval_ms: 50,
    };
    Box::into_raw(Box::new(plugin))
}
use redb::{Database, TableDefinition, ReadableTable};
use chrono::Utc;

const ORDER_STATES_TABLE: TableDefinition<&str, &str> = TableDefinition::new("order_states");
const CONTROL_EVENTS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("control_events");

pub struct JournalStorage { db: Database }

impl JournalStorage {
    pub fn new(db_path: &str) -> Self {
        let db = match Database::create(db_path) {
            Ok(db) => { tracing::info!("Execution journal opened at {}", db_path); db }
            Err(e) => {
                tracing::warn!("Failed to open journal at {}: {}. Attempting migration...", db_path, e);
                let backup = format!("{}.backup", db_path);
                let _ = std::fs::rename(db_path, &backup);
                Database::create(db_path).unwrap_or_else(|e2| panic!("Failed to create journal: {}", e2))
            }
        };
        Self { db }
    }

    pub fn append_order_state(&self, execution_id: &str, state: &str) -> Result<(), String> {
        let now = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let key = format!("{}:{}", now, execution_id);
        let txn = self.db.begin_write().map_err(|e| e.to_string())?;
        {
            let mut table = txn.open_table(ORDER_STATES_TABLE).map_err(|e| e.to_string())?;
            table.insert(key.as_str(), state).map_err(|e| e.to_string())?;
        }
        txn.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn append_control_event(&self, event_type: &str, payload: &str) -> Result<(), String> {
        let now = Utc::now().timestamp_millis();
        let key = format!("{}:{}", now, event_type);
        let txn = self.db.begin_write().map_err(|e| e.to_string())?;
        {
            let mut table = txn.open_table(CONTROL_EVENTS_TABLE).map_err(|e| e.to_string())?;
            table.insert(key.as_str(), payload).map_err(|e| e.to_string())?;
        }
        txn.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_latest_kill_switch_state(&self) -> Result<Option<bool>, String> {
        let txn = self.db.begin_read().map_err(|e| e.to_string())?;
        let table = txn.open_table(CONTROL_EVENTS_TABLE).map_err(|e| e.to_string())?;
        let mut latest: Option<bool> = None;
        for entry in table.iter().map_err(|e| e.to_string())? {
            let (_, value) = entry.map_err(|e| e.to_string())?;
            if let Ok(state) = serde_json::from_str::<bool>(value.value()) {
                latest = Some(state);
            }
        }
        Ok(latest)
    }

    pub fn get_latest_operation_mode(&self) -> Result<Option<String>, String> {
        let txn = self.db.begin_read().map_err(|e| e.to_string())?;
        let table = txn.open_table(CONTROL_EVENTS_TABLE).map_err(|e| e.to_string())?;
        let mut latest: Option<String> = None;
        for entry in table.iter().map_err(|e| e.to_string())? {
            let (_, value) = entry.map_err(|e| e.to_string())?;
            if let Ok(mode) = serde_json::from_str::<String>(value.value()) {
                latest = Some(mode);
            }
        }
        Ok(latest)
    }
}

impl Default for JournalStorage {
    fn default() -> Self {
        Self::new("execution_journal.db")
    }
}

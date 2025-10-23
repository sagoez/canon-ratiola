use crate::adapter::ClientRegistry;
use crate::domain::{
    AccountState, CommandMetadata, DeduplicationKey, OrchestratorMode, TransactionTypeCommand,
};
use std::collections::HashMap;
use std::fs::File;

pub struct Orchestrator {
    registry: ClientRegistry,
    mode: OrchestratorMode,
}

impl Orchestrator {
    pub async fn new(mode: OrchestratorMode) -> Self {
        let registry = super::boot().await;
        Self { registry, mode }
    }

    /// Create an Orchestrator with a custom registry.
    ///
    /// ## Warning: This is NOT MEANT FOR PRODUCTION USE. Only for testing purposes.
    pub fn with_registry(registry: ClientRegistry, mode: OrchestratorMode) -> Self {
        Self { registry, mode }
    }

    pub async fn process(self) -> Result<HashMap<u16, AccountState>, Box<dyn std::error::Error>> {
        let OrchestratorMode::Csv { file_path } = self.mode.clone();
        self.process_csv(&file_path).await
    }

    async fn process_csv(
        self,
        file_path: &str,
    ) -> Result<HashMap<u16, AccountState>, Box<dyn std::error::Error>> {
        let file_handle = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(file_handle);

        let mut line_num = 0;

        for result in rdr.deserialize() {
            line_num += 1;
            let command: TransactionTypeCommand = result?;
            let client_id = command.client_id();

            let metadata = CommandMetadata {
                deduplication_key: DeduplicationKey::new(format!("csv:{}:{}", file_path, line_num)),
            };

            // Process command via registry - it will get_or_spawn the client actor
            match self
                .registry
                .process_command(client_id, command, metadata)
                .await
            {
                Ok(_) => {}
                Err(e) => eprintln!("Error processing line {}: {}", line_num, e),
            }
        }

        // Give actors time to process all messages
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Collect all client states from the registry
        let states = self.registry.get_all_states().await?;

        // Shutdown all client actors
        self.registry.shutdown_all().await;

        Ok(states)
    }

    /// Output account states as CSV to stdout
    /// Writes one row per client, sorted by client_id
    pub fn output_csv(
        states: &HashMap<u16, AccountState>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        wtr.write_record(["client", "available", "held", "total", "locked"])?;

        // Sort by client_id for deterministic output
        let mut client_ids: Vec<_> = states.keys().collect();
        client_ids.sort();

        for client_id in client_ids {
            let state = &states[client_id];

            match state {
                AccountState::Active(s) => {
                    wtr.write_record([
                        &client_id.to_string(),
                        &format!("{:.4}", s.available),
                        &format!("{:.4}", s.held),
                        &format!("{:.4}", s.total),
                        "false",
                    ])?;
                }
                AccountState::Frozen(s) => {
                    wtr.write_record([
                        &client_id.to_string(),
                        &format!("{:.4}", s.available),
                        &format!("{:.4}", s.held),
                        &format!("{:.4}", s.total),
                        "true",
                    ])?;
                }
            }
        }

        wtr.flush()?;
        Ok(())
    }
}

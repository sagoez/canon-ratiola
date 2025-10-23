use rand::Rng;
use rand::seq::SliceRandom;
use std::fs::File;

/// Generate a mock CSV file with random transactions. This is used to test the payment system.
pub fn generator(output: &str, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output)?;
    let mut wtr = csv::Writer::from_writer(file);
    wtr.write_record(["type", "client", "tx", "amount"])?;

    let num_clients = (count / 10).clamp(10, 1000); // 10-1000 clients
    let transactions_per_client = count / num_clients;

    let mut rng = rand::rng();
    let mut tx_counter = 0u32;
    let mut all_transactions = Vec::new();

    for client_id in 1..=num_clients {
        let mut client_txs = Vec::new();

        let num_deposits = transactions_per_client / 2;
        let first_tx = tx_counter;

        for _ in 0..num_deposits {
            client_txs.push((
                "deposit",
                client_id,
                tx_counter,
                Some(1000.0 + rng.random_range(0.0..500.0)),
            ));
            tx_counter += 1;
        }

        if num_deposits > 0 {
            let num_withdrawals = (transactions_per_client / 4).max(1);
            for _i in 0..num_withdrawals {
                client_txs.push((
                    "withdrawal",
                    client_id,
                    tx_counter,
                    Some(rng.random_range(50.0..300.0)),
                ));
                tx_counter += 1;
            }
        }

        if client_id % 3 == 0 && num_deposits > 0 {
            let disputed_tx = first_tx + rng.random_range(0..num_deposits.min(1) as u32);
            client_txs.push(("dispute", client_id, disputed_tx, None));

            if client_id % 6 == 0 {
                client_txs.push(("chargeback", client_id, disputed_tx, None));
            } else {
                client_txs.push(("resolve", client_id, disputed_tx, None));
            }
        }

        all_transactions.extend(client_txs);
    }

    all_transactions.shuffle(&mut rng);

    for (tx_type, client_id, tx_id, amount) in &all_transactions {
        let client_str = client_id.to_string();
        let tx_str = tx_id.to_string();
        let amount_str = amount.map(|a| format!("{:.4}", a)).unwrap_or_default();

        wtr.write_record([*tx_type, &client_str, &tx_str, &amount_str])?;
    }

    wtr.flush()?;
    println!(
        "✓ Generated {} transactions across {} clients to {} (interleaved for concurrency testing)",
        all_transactions.len(),
        num_clients,
        output
    );
    Ok(())
}

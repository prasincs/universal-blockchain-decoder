#[cfg(test)]
mod test {
    use decoder_evm::ChainRegistry;

    #[test]
    fn check_exchange_chains() {
        let registry = ChainRegistry::new();

        // Chains supported by Binance, Crypto.com, Coinbase, Kraken
        let cronos = registry.get_chain(25);
        let fantom = registry.get_chain(250);
        let gnosis = registry.get_chain(100);
        let ronin = registry.get_chain(2020);

        println!("\n=== Exchange-Supported Chains Status ===");
        println!("Cronos (25): {:?}", cronos.is_some());
        println!("Fantom (250): {:?}", fantom.is_some());
        println!("Gnosis (100): {:?}", gnosis.is_some());
        println!("Ronin (2020): {:?}", ronin.is_some());

        if let Some(info) = cronos {
            println!("  Cronos: {}", info.name);
        }
        if let Some(info) = fantom {
            println!("  Fantom: {}", info.name);
        }
        if let Some(info) = gnosis {
            println!("  Gnosis: {}", info.name);
        }
        if let Some(info) = ronin {
            println!("  Ronin: {}", info.name);
        }

        println!("Total chains: {}", registry.all_chain_ids().len());
    }
}

pub mod actions;
pub mod agent;
pub mod benchmark;
pub mod env;
pub mod metrics;
pub mod results;
pub mod solana_env;
pub mod trace;

#[cfg(test)]
mod tests {
    use crate::benchmark::TestCase;

    #[test]
    fn it_loads_benchmark() {
        let benchmark_file = "../../benchmarks/001-sol-transfer.yml";
        let f = std::fs::File::open(benchmark_file).unwrap();
        let test_case: TestCase = serde_yaml::from_reader(f).unwrap();
        assert_eq!(test_case.id, "001-SOL-TRANSFER");
        assert_eq!(test_case.initial_state.len(), 2);
    }
}

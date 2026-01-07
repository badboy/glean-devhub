use std::fs;

use serde::Deserialize;
use xshell::{Shell, cmd};

use super::{Metric, MetricRecorder, Result};

const JQ_SCRIPT: &str = r#"
  .profiles[0].summaries.parts[0].metrics_summary.Callgrind as $callgrind
| .function_name + " " + .details as $name
| [ "Ir", "EstimatedCycles", "TotalRW", "L1hits", "LLhits", "RamHits" ] as $keys
| $keys
| map({
  name: $name + " -- " + .,
  value: $callgrind[.].metrics.Both[0].Int
})
"#;

/// Parse Gungraun benchmark results
pub struct Benchmark;

impl MetricRecorder for Benchmark {
    fn name(&self) -> &'static str {
        "benchmark"
    }

    fn description(&self) -> &'static str {
        "Gungraun Benchmark Results"
    }

    fn record(&self, sh: &Shell) -> Result<Vec<Metric>> {
        let temp = sh.create_temp_dir()?;

        let jq_script_path = temp.path().join("transform-gungraun.jq");
        sh.write_file(&jq_script_path, JQ_SCRIPT)?;

        let dest = sh.current_dir().join("gungraun-output.json");

        let mut metrics = Vec::new();

        let benchmarks = cmd!(sh, "jq -cf {jq_script_path} {dest}").read()?;
        let benchmarks = benchmarks.lines();
        for line in benchmarks {
            let bench: Vec<Bench> = serde_json::from_str(line)?;

            for b in bench {
                metrics.push(Metric {
                    name: b.name,
                    unit: String::from(""),
                    value: b.value,
                })
            }
        }

        Ok(metrics)
    }
}

#[derive(Deserialize)]
struct Bench {
    name: String,
    value: u64,
}

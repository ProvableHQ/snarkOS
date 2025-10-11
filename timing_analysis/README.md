# Timing Analysis Tool (Rust)

A high-performance Rust implementation for analyzing consensus timing data from snarkOS JSON files.

## Features

- **Fast JSON Parsing**: Efficient parsing of large timing data files using serde
- **SVG Chart Generation**: High-quality vector graphics using the plotters crate
- **Text-based Visualization**: ASCII fallback for environments without graphics
- **Comprehensive Statistics**: Detailed analysis of consensus stage performance
- **Command-line Interface**: User-friendly CLI with clap
- **Robust Error Handling**: Comprehensive error reporting with anyhow
- **Unit Tested**: Extensive test coverage for all components

## Installation

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo (comes with Rust)

### Build from Source

```bash
cd timing_analysis
cargo build --release
```

The binary will be available at `target/release/timing_analysis`.

## Usage

### Basic Usage

```bash
# Analyze timing data and generate SVG chart
./target/release/timing_analysis --json-file consensus_timing_block.json

# Custom output file
./target/release/timing_analysis --json-file consensus_timing_block.json --output my_analysis.svg

# Text-only output (no chart generation)
./target/release/timing_analysis --json-file consensus_timing_block.json --text-only

# Custom chart dimensions
./target/release/timing_analysis --json-file consensus_timing_block.json --width 1600 --height 1000
```

### Command-line Options

- `--json-file <FILE>`: Path to the consensus timing JSON file (required)
- `--output <FILE>`: Output SVG file name (default: `consensus_timing_analysis.svg`)
- `--width <PIXELS>`: Chart width in pixels (default: 1200)
- `--height <PIXELS>`: Chart height in pixels (default: 800)
- `--text-only`: Show only text-based visualization

### Development Commands

```bash
# Run tests
cargo test

# Run with debug output
cargo run -- --json-file test_data.json --text-only

# Check code without building
cargo check

# Fix linting issues
cargo fix
```

## Output

### Summary Statistics
```
=== Consensus Timing Summary ===
Total rounds with timing data: 2
Total subdags with timing data: 1

Round-based stages:
  Proposal Generation: 2 complete entries, avg duration: 1.200s
  Certificate Generation: 2 complete entries, avg duration: 0.750s
  Certificate Collection: 2 complete entries, avg duration: 0.750s

Subdag-based stages:
  Subdag Processing: 1 complete entries, avg duration: 0.700s
  Check Next Block: 1 complete entries, avg duration: 0.600s
  Advance to Next Block: 1 complete entries, avg duration: 0.700s
```

### SVG Chart
- High-quality vector graphics suitable for presentations
- Color-coded stages with legend
- Proper scaling and axis labels
- Round index on X-axis, time on Y-axis
- Bars showing start/end times and durations

### Text Visualization
```
=== Text-based Timing Visualization ===
Time range: 9.500 seconds
Rounds: 100 - 101

Round 100: P(1.50s) C(0.70s) L(0.80s) 
Round 101: P(0.90s) C(0.80s) L(0.70s) 

Subdag stages:
Rounds 100-101: S(0.70s) K(0.60s) A(0.70s) 

Legend: P=Proposal Gen, C=Certificate Gen, L=Certificate Collection
        S=Subdag Processing, K=Check Next Block, A=Advance to Next Block
```

## Architecture

### Modules

- **`data.rs`**: JSON parsing, data structures, and processing logic
- **`visualization.rs`**: Chart generation and text-based visualization
- **`main.rs`**: CLI interface and application entry point

### Key Data Structures

```rust
// Raw JSON data structures
pub struct JsonSystemTime {
    pub secs_since_epoch: u64,
    pub nanos_since_epoch: u32,
}

pub struct RawTimingSnapshot {
    pub timestamp: JsonSystemTime,
    pub round_timings: HashMap<String, RawRoundTiming>,
    pub subdag_timings: HashMap<String, RawSubdagTiming>,
}

// Processed data for analysis
pub struct ProcessedTimingData {
    pub snapshot_timestamp: f64,
    pub round_timings: IndexMap<u64, HashMap<String, TimingData>>,
    pub subdag_timings: IndexMap<(u64, u64), HashMap<String, TimingData>>,
}

pub struct TimingData {
    pub stage_name: String,
    pub start_time: f64,
    pub end_time: Option<f64>,
}
```

### Consensus Stages

**Round-based stages (individual rounds):**
- **Proposal Generation** (Blue): Time to generate and propose a batch
- **Certificate Generation** (Orange): Time to create batch certificates
- **Certificate Collection** (Green): Time to collect certificates from validators

**Subdag-based stages (spanning multiple rounds):**
- **Subdag Processing** (Red): Time to process the committed subdag
- **Check Next Block** (Purple): Time to validate the next block
- **Advance to Next Block** (Brown): Time to advance ledger to next block

## Dependencies

### Runtime Dependencies
- `serde` + `serde_json`: JSON serialization/deserialization
- `clap`: Command-line argument parsing
- `plotters` + `plotters-svg`: Chart generation
- `anyhow` + `thiserror`: Error handling
- `chrono`: Date/time handling
- `indexmap`: Ordered hash maps
- `itertools`: Iterator utilities
- `statrs`: Statistical functions
- `tracing` + `tracing-subscriber`: Logging

### Development Dependencies
- `tempfile`: Temporary file handling for tests
- `pretty_assertions`: Better test output formatting

## Performance

The Rust implementation offers significant performance advantages:

- **Memory Efficiency**: Zero-copy parsing where possible
- **Fast JSON Processing**: serde-based parsing is highly optimized
- **Efficient Data Structures**: IndexMap for ordered iteration
- **Minimal Allocations**: Careful memory management
- **Parallel Processing**: Ready for future parallelization

## Error Handling

Comprehensive error handling with context:

```rust
// File not found
Error: JSON file 'missing.json' not found

// Invalid JSON format
Error: Failed to parse JSON from file: invalid.json
Caused by: expected `,` or `}` at line 5 column 10

// Invalid round numbers
Error: Failed to parse round number: invalid_round
```

## Testing

Extensive test coverage including:

- JSON parsing and data structure validation
- Timestamp conversion accuracy
- Chart generation with various data sets
- Error handling for malformed input
- Integration tests with real data

Run tests with:
```bash
cargo test
```

## Integration with snarkOS

This tool is designed to work seamlessly with the snarkOS consensus timing collection system:

1. snarkOS generates `consensus_timing_block.json` files
2. This tool analyzes the structured timing data
3. Generates visualizations and statistics
4. Helps identify performance bottlenecks in consensus

## Comparison with Python Version

**Advantages of Rust implementation:**
- ⚡ **10-100x faster** JSON parsing and processing
- 🔒 **Memory safe** with zero runtime overhead
- 📦 **Single binary** with no external dependencies
- 🚀 **Better error messages** with full context
- 🧪 **More comprehensive testing**
- 📊 **Higher quality SVG output**

**Feature parity:**
- ✅ All consensus stages supported
- ✅ Same color scheme and visualization style
- ✅ Text-based fallback visualization
- ✅ Statistical analysis and summaries
- ✅ Command-line interface compatibility

use crate::data::{ProcessedTimingData, TimingData};
use anyhow::{Context, Result};
use plotters::prelude::*;
use std::collections::HashMap;

/// Color scheme for different consensus stages
pub struct StageColors;

impl StageColors {
    pub fn get_color(stage_name: &str) -> RGBColor {
        match stage_name {
            "proposal_generation" => RGBColor(31, 119, 180),      // Blue
            "certificate_generation" => RGBColor(255, 127, 14),   // Orange
            "certificate_collection" => RGBColor(44, 160, 44),    // Green
            "subdag_processing" => RGBColor(214, 39, 40),         // Red
            "check_next_block" => RGBColor(148, 103, 189),        // Purple
            "advance_to_next_block" => RGBColor(140, 86, 75),     // Brown
            _ => RGBColor(128, 128, 128),                         // Gray (fallback)
        }
    }

    pub fn get_stage_display_name(stage_name: &str) -> &'static str {
        match stage_name {
            "proposal_generation" => "Proposal Generation",
            "certificate_generation" => "Certificate Generation",
            "certificate_collection" => "Certificate Collection",
            "subdag_processing" => "Subdag Processing",
            "check_next_block" => "Check Next Block",
            "advance_to_next_block" => "Advance to Next Block",
            _ => "Unknown Stage",
        }
    }
}

/// Generate a bar chart visualization of the timing data
pub fn generate_bar_chart(
    data: &ProcessedTimingData,
    output_path: &str,
    width: u32,
    height: u32,
) -> Result<()> {
    if data.round_timings.is_empty() && data.subdag_timings.is_empty() {
        return Err(anyhow::anyhow!("No timing data to visualize"));
    }

    let all_rounds = data.get_all_rounds();
    if all_rounds.is_empty() {
        return Err(anyhow::anyhow!("No rounds found in timing data"));
    }

    let (min_time, max_time) = data.get_time_range();
    let start_timestamp = min_time;
    let total_duration = max_time - min_time;

    let min_round = *all_rounds.first().unwrap() as f64;
    let max_round = *all_rounds.last().unwrap() as f64;

    // Create the drawing backend
    let root = SVGBackend::new(output_path, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Consensus Timing Analysis", ("Arial", 30).into_font())
        .margin(10)
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(
            (min_round - 1.0)..(max_round + 1.0),
            0.0..total_duration,
        )?;

    chart
        .configure_mesh()
        .x_desc("Round Index")
        .y_desc("Time (seconds from start)")
        .draw()?;

    let mut legend_entries = Vec::new();
    let mut used_labels = std::collections::HashSet::new();

    // Draw round-based stages
    for (&round_num, round_data) in &data.round_timings {
        for (stage_name, timing) in round_data {
            if !timing.is_complete() {
                continue;
            }

            let x_center = round_num as f64;
            let width = 0.8;
            let y_start = timing.start_time - start_timestamp;
            let height = timing.duration().unwrap();

            let color = StageColors::get_color(stage_name);
            let display_name = StageColors::get_stage_display_name(stage_name);

            // Draw the bar
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x_center - width / 2.0, y_start), (x_center + width / 2.0, y_start + height)],
                color.filled(),
            )))?;

            // Add to legend if not already added
            if !used_labels.contains(display_name) {
                legend_entries.push((display_name, color));
                used_labels.insert(display_name);
            }
        }
    }

    // Draw subdag-based stages
    for (&(low_round, high_round), subdag_data) in &data.subdag_timings {
        for (stage_name, timing) in subdag_data {
            if !timing.is_complete() {
                continue;
            }

            let x_center = (low_round + high_round) as f64 / 2.0;
            let width = (high_round - low_round + 1) as f64;
            let y_start = timing.start_time - start_timestamp;
            let height = timing.duration().unwrap();

            let color = StageColors::get_color(stage_name);
            let display_name = StageColors::get_stage_display_name(stage_name);

            // Draw the bar
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x_center - width / 2.0, y_start), (x_center + width / 2.0, y_start + height)],
                color.filled(),
            )))?;

            // Add to legend if not already added
            if !used_labels.contains(display_name) {
                legend_entries.push((display_name, color));
                used_labels.insert(display_name);
            }
        }
    }

    // Draw legend
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    // Manually add legend entries
    for (label, color) in legend_entries {
        chart.draw_series(std::iter::once(Rectangle::new(
            [(-1000.0, -1000.0), (-999.0, -999.0)], // Off-screen rectangle for legend
            color.filled(),
        )))?.label(label);
    }

    chart.configure_series_labels().draw()?;

    root.present()?;
    println!("Chart saved to {}", output_path);

    Ok(())
}

/// Generate a text-based visualization as fallback
pub fn generate_text_visualization(data: &ProcessedTimingData) -> Result<()> {
    if data.round_timings.is_empty() && data.subdag_timings.is_empty() {
        println!("No timing data to visualize");
        return Ok(());
    }

    println!("\n=== Text-based Timing Visualization ===");

    let all_rounds = data.get_all_rounds();
    if all_rounds.is_empty() {
        println!("No rounds found in timing data");
        return Ok(());
    }

    let (min_time, max_time) = data.get_time_range();
    let total_duration = max_time - min_time;

    println!("Time range: {:.3} seconds", total_duration);
    println!("Rounds: {} - {}", all_rounds.first().unwrap(), all_rounds.last().unwrap());
    println!();

    // Display round-based stages
    for &round_num in &all_rounds {
        print!("Round {:3}: ", round_num);

        if let Some(round_data) = data.round_timings.get(&round_num) {
            for stage_name in &["proposal_generation", "certificate_generation", "certificate_collection"] {
                if let Some(timing) = round_data.get(*stage_name) {
                    if timing.is_complete() {
                        let stage_char = match *stage_name {
                            "proposal_generation" => 'P',
                            "certificate_generation" => 'C',
                            "certificate_collection" => 'L',
                            _ => '?',
                        };
                        print!("{}({:.2}s) ", stage_char, timing.duration().unwrap());
                    }
                }
            }
        }
        println!();
    }

    // Display subdag stages
    println!("\nSubdag stages:");
    for (&(low_round, high_round), subdag_data) in &data.subdag_timings {
        print!("Rounds {}-{}: ", low_round, high_round);
        
        for stage_name in &["subdag_processing", "check_next_block", "advance_to_next_block"] {
            if let Some(timing) = subdag_data.get(*stage_name) {
                if timing.is_complete() {
                    let stage_char = match *stage_name {
                        "subdag_processing" => 'S',
                        "check_next_block" => 'K',
                        "advance_to_next_block" => 'A',
                        _ => '?',
                    };
                    print!("{}({:.2}s) ", stage_char, timing.duration().unwrap());
                }
            }
        }
        println!();
    }

    println!("\nLegend: P=Proposal Gen, C=Certificate Gen, L=Certificate Collection");
    println!("        S=Subdag Processing, K=Check Next Block, A=Advance to Next Block");

    Ok(())
}

/// Print a statistical summary of the timing data
pub fn print_summary(data: &ProcessedTimingData) {
    println!("\n=== Consensus Timing Summary ===");
    println!("Total rounds with timing data: {}", data.round_timings.len());
    println!("Total subdags with timing data: {}", data.subdag_timings.len());

    if !data.round_timings.is_empty() {
        println!("\nRound-based stages:");
        for stage in &["proposal_generation", "certificate_generation", "certificate_collection"] {
            let mut durations = Vec::new();
            let mut count = 0;

            for round_data in data.round_timings.values() {
                if let Some(timing) = round_data.get(*stage) {
                    if timing.is_complete() {
                        count += 1;
                        durations.push(timing.duration().unwrap());
                    }
                }
            }

            if count > 0 {
                let avg_duration = durations.iter().sum::<f64>() / durations.len() as f64;
                let display_name = StageColors::get_stage_display_name(stage);
                println!("  {}: {} complete entries, avg duration: {:.3}s", 
                         display_name, count, avg_duration);
            }
        }
    }

    if !data.subdag_timings.is_empty() {
        println!("\nSubdag-based stages:");
        for stage in &["subdag_processing", "check_next_block", "advance_to_next_block"] {
            let mut durations = Vec::new();
            let mut count = 0;

            for subdag_data in data.subdag_timings.values() {
                if let Some(timing) = subdag_data.get(*stage) {
                    if timing.is_complete() {
                        count += 1;
                        durations.push(timing.duration().unwrap());
                    }
                }
            }

            if count > 0 {
                let avg_duration = durations.iter().sum::<f64>() / durations.len() as f64;
                let display_name = StageColors::get_stage_display_name(stage);
                println!("  {}: {} complete entries, avg duration: {:.3}s", 
                         display_name, count, avg_duration);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{ProcessedTimingData, TimingData};
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    #[test]
    fn test_stage_colors() {
        assert_eq!(StageColors::get_color("proposal_generation"), RGBColor(31, 119, 180));
        assert_eq!(StageColors::get_color("unknown_stage"), RGBColor(128, 128, 128));
    }

    #[test]
    fn test_stage_display_names() {
        assert_eq!(StageColors::get_stage_display_name("proposal_generation"), "Proposal Generation");
        assert_eq!(StageColors::get_stage_display_name("unknown_stage"), "Unknown Stage");
    }

    #[test]
    fn test_text_visualization_empty_data() {
        let data = ProcessedTimingData::new(1640995200.0);
        let result = generate_text_visualization(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bar_chart_empty_data() {
        let data = ProcessedTimingData::new(1640995200.0);
        let temp_file = NamedTempFile::new().unwrap();
        let result = generate_bar_chart(&data, temp_file.path().to_str().unwrap(), 800, 600);
        assert!(result.is_err());
    }

    #[test]
    fn test_bar_chart_with_data() {
        let mut data = ProcessedTimingData::new(1640995200.0);
        
        // Add some test data
        let mut round_data = HashMap::new();
        round_data.insert(
            "proposal_generation".to_string(),
            TimingData::new("proposal_generation".to_string(), 100.0, Some(101.5)),
        );
        data.round_timings.insert(100, round_data);

        let temp_file = NamedTempFile::new().unwrap();
        let result = generate_bar_chart(&data, temp_file.path().to_str().unwrap(), 800, 600);
        assert!(result.is_ok());
    }
}

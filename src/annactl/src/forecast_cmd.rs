//! Forecast command for Anna v0.14.0 "Orion III"
//!
//! Display predictive forecasts

use anyhow::Result;
use anna_common::{header, section, TermCaps};

use crate::forecast::{Forecast, ForecastEngine, ForecastWindow};

/// Run forecast command
pub fn run_forecast(seven_day: bool, thirty_day: bool, json: bool) -> Result<()> {
    let engine = ForecastEngine::new()?;

    let forecasts = if seven_day && thirty_day {
        vec![engine.forecast_7d()?, engine.forecast_30d()?]
    } else if thirty_day {
        vec![engine.forecast_30d()?]
    } else {
        vec![engine.forecast_7d()?] // Default to 7-day
    };

    if json {
        let json_str = serde_json::to_string_pretty(&forecasts)?;
        println!("{}", json_str);
        return Ok(());
    }

    print_forecasts(&forecasts);

    Ok(())
}

/// Print forecasts with TUI
fn print_forecasts(forecasts: &[Option<Forecast>]) {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "Predictive Forecasts"));
    println!();

    let mut has_forecast = false;

    for forecast_opt in forecasts {
        if let Some(forecast) = forecast_opt {
            has_forecast = true;

            let window_name = if forecast.window_days == 7 {
                "7-Day"
            } else {
                "30-Day"
            };

            println!("{}", section(&caps, &format!("{} Forecast", window_name)));
            println!();

            // Trajectory
            let emoji = ForecastEngine::trajectory_emoji(&forecast.trajectory);
            let color = ForecastEngine::trajectory_color(&forecast.trajectory);

            println!("  Trajectory:  {}{}  {}\x1b[0m",
                     color,
                     emoji,
                     forecast.trajectory);
            println!();

            // Scores
            println!("  Current Score:    {:.1}/10", forecast.current_score);
            println!("  Predicted Score:  {:.1}/10", forecast.predicted_score);
            println!("  Confidence Range: [{:.1}, {:.1}]",
                     forecast.confidence_lower,
                     forecast.confidence_upper);
            println!();

            // Statistics
            println!("  Moving Average:   {:.1}", forecast.moving_average);
            println!("  Exp. Weighted:    {:.1} (α=0.5)", forecast.exponential_weighted);
            println!();

            // Deviation
            let deviation_color = if forecast.deviation.abs() < 0.5 {
                "\x1b[32m" // Green - on track
            } else if forecast.deviation.abs() < 1.5 {
                "\x1b[33m" // Yellow - minor deviation
            } else {
                "\x1b[31m" // Red - significant deviation
            };

            println!("  Deviation:        {}{:+.1}\x1b[0m (actual vs predicted)",
                     deviation_color,
                     forecast.deviation);
            println!();

            // Interpretation
            print_interpretation(forecast);
            println!();
        }
    }

    if !has_forecast {
        println!("  Insufficient historical data for forecasting");
        println!();
        println!("  Run 'annactl report' multiple times over several days");
        println!("  to build up history for trend analysis.");
    }
}

/// Print forecast interpretation
fn print_interpretation(forecast: &Forecast) {
    println!("  Interpretation:");
    println!();

    match forecast.trajectory.as_str() {
        "improving" => {
            println!("    ✅ System health is trending upward");
            println!("    Expected to improve over next {} days", forecast.window_days);
        }
        "degrading" => {
            println!("    ⚠️  System health is trending downward");
            println!("    Review recommendations: annactl report --verbose");
        }
        _ => {
            println!("    → System health is stable");
            println!("    No significant change expected");
        }
    }

    if forecast.deviation.abs() > 1.5 {
        println!();
        println!("    ⚠️  Current score deviates significantly from prediction");
        println!("    Consider investigating recent changes");
    }
}

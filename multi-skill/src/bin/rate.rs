extern crate multi_skill;

use multi_skill::data_processing::get_dataset_by_name;
use multi_skill::experiment_config::Experiment;
use multi_skill::summary::print_ratings;
use multi_skill::systems::{get_rating_system_by_name, simulate_contest};
use std::collections::HashMap;

fn get_experiment_from_args(args: &[String]) -> Experiment {
    if args[1] == "file:" {
        return Experiment::from_file(&args[2]);
    }

    let system = get_rating_system_by_name(&args[1]).unwrap();
    let dataset = get_dataset_by_name(&args[2]).unwrap();
    let max_contests = args
        .get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(usize::MAX);

    Experiment {
        max_contests,
        mu_noob: 1500.,
        sig_noob: 350.,
        system,
        dataset,
    }
}

/// Simulates the entire history of Codeforces
fn main() {
    // Parse arguments, prepare rating system and datasets
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 && args.len() != 4 {
        eprintln!("Usage: {} system_name dataset_name [num_contests]", args[0]);
        return;
    }
    let ex = get_experiment_from_args(&args);

    // Simulate the contests and rating updates
    let mut players = HashMap::new();
    let mut last_contest_time = 0;
    for (idx, contest) in ex.dataset.iter().enumerate().take(ex.max_contests) {
        println!(
            "Processing{:6} contestants in{:5}th contest with id={:4}, wt={}: {}",
            contest.standings.len(),
            idx,
            contest.id,
            contest.weight,
            contest.name
        );
        simulate_contest(&mut players, &contest, &*ex.system, ex.mu_noob, ex.sig_noob);
        last_contest_time = contest.time_seconds;
    }
    let six_months_ago = last_contest_time.saturating_sub(183 * 86_400);

    // Print ratings list to data/codeforces/CFratings.txt
    print_ratings(&players, six_months_ago);

    // Print contest histories of top players to data/codeforces/top/{handle}.json
    let dir = std::path::PathBuf::from("../data/codeforces/top");
    std::fs::create_dir_all(&dir).expect("Could not create directory");
    for (handle, player) in &players {
        let player = player.borrow();
        let last_event = player.event_history.last().expect("Empty history");

        if last_event.display_rating >= 2700 && last_event.contest_time > six_months_ago {
            let file = dir.join(format!("{}.json", handle));
            let data_rust = &player.event_history;
            let data_json = serde_json::to_string_pretty(&data_rust).expect("Serialization error");
            std::fs::write(&file, data_json).expect("Failed to write to cache");
            println!("Wrote to {:?}", file);
        }
    }
}

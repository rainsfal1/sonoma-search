use std::collections::HashMap;
use uuid::Uuid;

pub fn display_rank_info(ranks: &HashMap<Uuid, f64>) {
    let mut sorted_ranks: Vec<_> = ranks.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_ranks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\nDetailed Page Ranks:");
    println!("Total Pages: {}", sorted_ranks.len());
    println!("Rank Details:");
    println!("{:<40} {:<20} {:<20}", "Page ID", "Rank Score", "Normalized Percentage");
    println!("{:-<80}", "");

    let max_rank = sorted_ranks.first().map(|(_, r)| *r).unwrap_or(1.0);

    for (page, rank) in &sorted_ranks {
        let percentage = (rank / max_rank) * 100.0;
        println!(
            "{:<40} {:<20.6} {:<.2}%",
            page,
            rank,
            percentage
        );
    }
}

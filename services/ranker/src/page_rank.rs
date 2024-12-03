use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::data_models::LinkStorage;
use log::info;

pub fn prepare_page_links(rows: Vec<LinkStorage>) -> (HashMap<Uuid, Vec<Uuid>>, HashSet<Uuid>) {
    let mut page_links: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    let mut unique_pages: HashSet<Uuid> = HashSet::new();

    for row in rows {
        let source = row.source_webpage_id;
        let target = row.target_url;

        let target_uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, target.as_bytes());

        unique_pages.insert(source);
        unique_pages.insert(target_uuid);

        page_links.entry(source)
            .or_insert_with(Vec::new)
            .push(target_uuid);
    }

    // Remove duplicate links while preserving order
    for links in page_links.values_mut() {
        links.sort_unstable();
        links.dedup();
    }

    info!("Prepared {} pages with links", page_links.len());
    info!("Total unique pages: {}", unique_pages.len());

    (page_links, unique_pages)
}

pub fn calculate_page_rank(
    links: &HashMap<Uuid, Vec<Uuid>>,
    damping_factor: f64,
    iterations: usize,
) -> HashMap<Uuid, f64> {
    let mut all_pages = HashSet::new();
    for (source, targets) in links.iter() {
        all_pages.insert(*source);
        all_pages.extend(targets);
    }
    
    let num_pages = all_pages.len();
    if num_pages == 0 {
        return HashMap::new();
    }

    info!("Starting PageRank calculation for {} pages", num_pages);

    // Initialize all pages with equal rank
    let initial_rank = 1.0 / num_pages as f64;
    let mut ranks: HashMap<Uuid, f64> = all_pages
        .iter()
        .map(|page| (*page, initial_rank))
        .collect();

    // Identify pages with no outgoing links
    let dangling_nodes: Vec<_> = all_pages.iter()
        .filter(|&page| !links.contains_key(page))
        .collect();

    for _ in 0..iterations {
        let mut new_ranks = HashMap::with_capacity(num_pages);
        
        // Initialize with damping factor distribution
        let base_rank = (1.0 - damping_factor) / num_pages as f64;
        for &page in all_pages.iter() {
            new_ranks.insert(page, base_rank);
        }

        // Add rank from regular pages
        for (source, targets) in links {
            let out_degree = targets.len();
            if out_degree > 0 {
                let rank_share = ranks[source] / out_degree as f64;
                for target in targets {
                    *new_ranks.get_mut(target).unwrap() += damping_factor * rank_share;
                }
            }
        }

        // Add rank from dangling nodes
        let dangling_rank = dangling_nodes.iter()
            .map(|&page| ranks[page])
            .sum::<f64>() * damping_factor / num_pages as f64;

        for rank in new_ranks.values_mut() {
            *rank += dangling_rank;
        }

        ranks = new_ranks;
    }

    // Log some statistics about the final ranks
    let mut rank_values: Vec<f64> = ranks.values().copied().collect();
    rank_values.sort_by(|a, b| b.partial_cmp(a).unwrap());
    
    if !rank_values.is_empty() {
        info!("Top 5 ranks: {:?}", &rank_values[..5.min(rank_values.len())]);
        info!("Average rank: {}", rank_values.iter().sum::<f64>() / rank_values.len() as f64);
    }

    ranks
}
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::data_models::LinkStorage;

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

    (page_links, unique_pages)
}

pub fn calculate_page_rank(
    links: &HashMap<Uuid, Vec<Uuid>>,
    damping_factor: f64,
    iterations: usize,
) -> HashMap<Uuid, f64> {
    let num_pages = links.len();
    let initial_rank = 1.0 / num_pages as f64;

    let pages_without_outgoing: Vec<Uuid> = links
        .iter()
        .filter(|(_, targets)| targets.is_empty())
        .map(|(page, _)| *page)
        .collect();

    let mut ranks: HashMap<Uuid, f64> = links.keys().map(|page| (*page, initial_rank)).collect();

    for _ in 0..iterations {
        let mut next_ranks = ranks.clone();

        for (page, outgoing_links) in links.iter() {
            if outgoing_links.is_empty() {
                continue;
            }

            let rank_share = ranks[page] / outgoing_links.len() as f64;

            for target in outgoing_links {
                *next_ranks.entry(*target).or_insert(0.0) += damping_factor * rank_share;
            }
        }

        let dangling_rank_share: f64 = pages_without_outgoing
            .iter()
            .map(|page| ranks[page])
            .sum::<f64>() * damping_factor / num_pages as f64;

        for rank in next_ranks.values_mut() {
            *rank += (1.0 - damping_factor) / num_pages as f64;
            *rank += dangling_rank_share;
        }

        let total_rank: f64 = next_ranks.values().sum();
        for rank in next_ranks.values_mut() {
            *rank /= total_rank;
            *rank *= 200.0;
        }

        ranks = next_ranks;
    }

    ranks
}

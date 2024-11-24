use std::collections::HashMap;
use std::cmp::Ordering;

pub fn tfidf_summary(content: &str, summary_length: usize) -> String {
    let sentences: Vec<&str> = content.split('.').collect();
    
    let mut word_freq = HashMap::new();
    let mut word_sentences = HashMap::new();
    
    for (i, &sentence) in sentences.iter().enumerate() {
        let sentence_words: Vec<&str> = sentence.split_whitespace().collect();
        for word in &sentence_words {
            *word_freq.entry(word.to_lowercase()).or_insert(0) += 1;
            word_sentences.entry(word.to_lowercase()).or_insert(Vec::new()).push(i);
        }
    }
    
    let total_sentences = sentences.len() as f64;
    
    let mut sentence_scores: Vec<(usize, f64)> = sentences.iter().enumerate()
        .map(|(i, &sentence)| {
            let score = sentence.split_whitespace()
                .map(|word| {
                    let tf = *word_freq.get(&word.to_lowercase()).unwrap_or(&0) as f64;
                    let idf = (total_sentences / word_sentences.get(&word.to_lowercase()).map_or(1.0, |v| v.len() as f64)).ln();
                    tf * idf
                })
                .sum::<f64>() / sentence.split_whitespace().count().max(1) as f64;
            (i, score)
        })
        .collect();
    
    sentence_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    
    let summary_length = summary_length.min(sentences.len());
    
    sentence_scores.iter()
        .take(summary_length)
        .map(|&(i, _)| sentences[i])
        .collect::<Vec<&str>>()
        .join(". ")
}

//tokenizer.rs

use tantivy::tokenizer::{SimpleTokenizer, RemoveLongFilter, LowerCaser, TextAnalyzer};

pub fn tokenizer() -> TextAnalyzer {
    let tokenizer = SimpleTokenizer
        .filter(RemoveLongFilter::limit(40))
        .filter(LowerCaser);
    TextAnalyzer::from(tokenizer)
}
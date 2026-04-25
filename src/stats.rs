pub struct UsageEvent {
    pub ts: String,
    pub query: String,
    pub query_len: usize,
    pub result_count: usize,
    pub hit: bool,
    pub used_type: bool,
    pub used_lang: bool,
    pub used_path: bool,
    pub used_limit: bool,
    pub repo: String,
    pub indexed_files: usize,
}
pub struct UsageSummary {
    pub total: usize,
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
    pub avg_results: f64,
    pub estimated_blind_reads_avoided: usize,
    pub estimated_tokens_avoided: usize,
}

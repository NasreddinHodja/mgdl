use serde::Serialize;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct BenchCollector {
    start: Instant,
    events: Arc<Mutex<Vec<BenchEvent>>>,
}

enum BenchEvent {
    ScrapeComplete { duration: Duration },
    ChapterDiscovered { duration: Duration },
    ChapterSkipped,
    PageDownloaded { duration: Duration, bytes: usize },
    PageSkipped,
}

#[derive(Serialize)]
pub struct BenchReport {
    pub manga_name: String,
    pub total_time_secs: f64,
    pub scrape_time_secs: f64,
    pub chapters_discovered: usize,
    pub chapters_skipped: usize,
    pub chapter_discovery_secs: f64,
    pub pages_downloaded: usize,
    pub pages_skipped: usize,
    pub total_bytes: usize,
    pub avg_page_time_ms: f64,
    pub min_page_time_ms: f64,
    pub max_page_time_ms: f64,
    pub pages_per_sec: f64,
    pub megabytes_per_sec: f64,
}

impl BenchCollector {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn record(&self, event: BenchEvent) {
        self.events.lock().unwrap().push(event);
    }

    pub fn record_scrape(&self, duration: Duration) {
        self.record(BenchEvent::ScrapeComplete { duration });
    }

    pub fn record_chapter_discovered(&self, duration: Duration) {
        self.record(BenchEvent::ChapterDiscovered { duration });
    }

    pub fn record_chapter_skipped(&self) {
        self.record(BenchEvent::ChapterSkipped);
    }

    pub fn record_page_downloaded(&self, duration: Duration, bytes: usize) {
        self.record(BenchEvent::PageDownloaded { duration, bytes });
    }

    pub fn record_page_skipped(&self) {
        self.record(BenchEvent::PageSkipped);
    }

    pub fn finish(self, manga_name: &str) -> BenchReport {
        let total_time = self.start.elapsed();
        let events = self.events.lock().unwrap();

        let mut scrape_time = Duration::ZERO;
        let mut chapters_discovered = 0usize;
        let mut chapters_skipped = 0usize;
        let mut chapter_discovery_time = Duration::ZERO;
        let mut pages_downloaded = 0usize;
        let mut pages_skipped = 0usize;
        let mut total_bytes = 0usize;
        let mut page_times: Vec<Duration> = Vec::new();

        for event in events.iter() {
            match event {
                BenchEvent::ScrapeComplete { duration } => scrape_time = *duration,
                BenchEvent::ChapterDiscovered { duration } => {
                    chapters_discovered += 1;
                    chapter_discovery_time += *duration;
                }
                BenchEvent::ChapterSkipped => chapters_skipped += 1,
                BenchEvent::PageDownloaded { duration, bytes } => {
                    pages_downloaded += 1;
                    total_bytes += bytes;
                    page_times.push(*duration);
                }
                BenchEvent::PageSkipped => pages_skipped += 1,
            }
        }

        let (avg_page_time_ms, min_page_time_ms, max_page_time_ms) = if page_times.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            let sum: Duration = page_times.iter().sum();
            let avg = sum.as_secs_f64() * 1000.0 / page_times.len() as f64;
            let min = page_times.iter().min().unwrap().as_secs_f64() * 1000.0;
            let max = page_times.iter().max().unwrap().as_secs_f64() * 1000.0;
            (avg, min, max)
        };

        let total_secs = total_time.as_secs_f64();
        let pages_per_sec = if total_secs > 0.0 {
            pages_downloaded as f64 / total_secs
        } else {
            0.0
        };
        let megabytes_per_sec = if total_secs > 0.0 {
            (total_bytes as f64 / (1024.0 * 1024.0)) / total_secs
        } else {
            0.0
        };

        BenchReport {
            manga_name: manga_name.to_string(),
            total_time_secs: total_secs,
            scrape_time_secs: scrape_time.as_secs_f64(),
            chapters_discovered,
            chapters_skipped,
            chapter_discovery_secs: chapter_discovery_time.as_secs_f64(),
            pages_downloaded,
            pages_skipped,
            total_bytes,
            avg_page_time_ms,
            min_page_time_ms,
            max_page_time_ms,
            pages_per_sec,
            megabytes_per_sec,
        }
    }
}

impl BenchReport {
    pub fn print_summary(&self) {
        let total_mb = self.total_bytes as f64 / (1024.0 * 1024.0);
        eprintln!("──── Benchmark Results ────────────────────────");
        eprintln!("Manga:           {}", self.manga_name);
        eprintln!("Total time:      {:.1}s", self.total_time_secs);
        eprintln!("Scrape time:     {:.1}s", self.scrape_time_secs);
        eprintln!(
            "Chapters:        {} ({} skipped)",
            self.chapters_discovered, self.chapters_skipped
        );
        eprintln!(
            "Pages:           {} downloaded, {} skipped",
            self.pages_downloaded, self.pages_skipped
        );
        eprintln!(
            "Download time:   avg {:.0}ms  min {:.0}ms  max {:.0}ms",
            self.avg_page_time_ms, self.min_page_time_ms, self.max_page_time_ms
        );
        eprintln!(
            "Throughput:      {:.1} pages/sec  |  {:.1} MB/s",
            self.pages_per_sec, self.megabytes_per_sec
        );
        eprintln!("Total size:      {:.1} MB", total_mb);
        eprintln!("───────────────────────────────────────────────");
    }

    pub fn write_json(&self, dir: &Path) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let path = dir.join(format!("bench_{}.json", timestamp));
        match serde_json::to_string_pretty(self) {
            Ok(json) => match std::fs::write(&path, json) {
                Ok(()) => eprintln!("Report saved to {}", path.display()),
                Err(e) => eprintln!("Failed to write bench report: {}", e),
            },
            Err(e) => eprintln!("Failed to serialize bench report: {}", e),
        }
    }
}

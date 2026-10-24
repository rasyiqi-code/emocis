pub mod audit;
pub mod context;
pub mod duplication;
pub mod failed;
pub mod health;
pub mod init;
pub mod knowledge;
pub mod map;
pub mod refactor;
pub mod search;
pub mod snapshot;

use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Inisialisasi engine pada repo (.emocis/)
    Init,

    /// Memetakan struktur, komentar, keputusan & relasi repo ke knowledge graph
    Map {
        /// Force full re-indexing (abaikan cache content-hash)
        #[arg(short, long)]
        force: bool,
    },

    /// Menampilkan Conceptual Table of Contents (TOC)
    Knowledge,

    /// Pencarian kata kunci untuk keputusan teknis, batasan, dan peringatan
    Search {
        /// Kata kunci pencarian
        query: String,
    },

    /// Menghasilkan snapshot kondisi riil proyek terkini
    Snapshot,

    /// Menghitung skor kesehatan arsitektur codebase eksperimental (skor belum tersedia)
    Health,

    /// Menghasilkan Minimal Context Package untuk AI Coding Agent (hemat token)
    Context {
        /// Target intent atau tugas yang akan dikerjakan AI
        intent: String,
    },

    /// Menjalankan audit deteksi drift arsitektur dan dokumentasi usang
    Audit,

    /// Sinyal ukuran, kompleksitas AST per fungsi, dan leksikal untuk tinjauan manual
    Refactor {
        /// File target (opsional, jika kosong memindai seluruh proyek)
        file: Option<String>,
    },

    /// Mengelola memori kegagalan masa lalu (Failed Attempts) agar tidak diulangi developer / AI
    Failed {
        /// Pendekatan yang pernah dicoba (misal: "Redis-based distributed lock")
        #[arg(long)]
        approach: Option<String>,
        /// Alasan penolakan / kegagalan
        #[arg(long)]
        reason: Option<String>,
        /// Solusi pengganti yang berhasil
        #[arg(long)]
        replaced_by: Option<String>,
        /// Provenance (PR atau commit id)
        #[arg(long)]
        provenance: Option<String>,
    },

    /// Mencari kandidat kemiripan nama dan pola panggilan antar modul
    Duplication,
}

pub fn run(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Init => init::execute(),
        Commands::Map { force } => map::execute(force),
        Commands::Knowledge => knowledge::execute(),
        Commands::Search { query } => search::execute(&query),
        Commands::Snapshot => snapshot::execute(),
        Commands::Health => health::execute(),
        Commands::Context { intent } => context::execute(&intent),
        Commands::Audit => audit::execute(),
        Commands::Refactor { file } => refactor::execute(file),
        Commands::Failed {
            approach,
            reason,
            replaced_by,
            provenance,
        } => failed::execute(approach, reason, replaced_by, provenance),
        Commands::Duplication => duplication::execute(),
    }
}

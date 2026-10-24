use emocis_core::KnowledgeType;
use emocis_memory::extractor::KnowledgeExtractor;

#[test]
fn test_multiline_slash_slash_aggregation() {
    let extractor = KnowledgeExtractor::new();
    let code = r#"
fn process_invoice() {
    // BUG FIX: Race condition pada invoice sequence number saat simultaneous checkout.
    // Kita harus menggunakan transaction lock atau atomik INCREMENT di SQLite
    // agar nomor nota tidak duplikat saat multi-kasir.
    let invoice_id = get_next_sequence();
}
"#;

    let items = extractor.extract_from_code("kasir_project", "src/invoice.rs", code);
    assert_eq!(items.len(), 1);
    let item = &items[0];

    assert_eq!(item.knowledge_type, KnowledgeType::Warning);
    assert!(item.tags.contains(&"bugfix".to_string()));
    assert!(item.tags.contains(&"bug fix".to_string()));
    assert!(item.title.contains("BUG FIX"));

    // Check that multi-line content was aggregated
    assert!(item.content.contains("Race condition pada invoice sequence number"));
    assert!(item.content.contains("Kita harus menggunakan transaction lock"));
    assert!(item.content.contains("agar nomor nota tidak duplikat"));

    // Check evidence
    let ev = &item.evidence[0];
    assert_eq!(ev.start_line, Some(3));
    assert_eq!(ev.end_line, Some(5));
    assert!(ev.snippet.as_ref().unwrap().contains("// BUG FIX:"));
    assert!(ev.snippet.as_ref().unwrap().contains("// agar nomor"));
}

#[test]
fn test_python_hash_aggregation() {
    let extractor = KnowledgeExtractor::new();
    let code = r#"
# ALASAN: Endpoint ini dibatasi rate limit 5 req/s
# karena upstream API vendor bank memiliki limitasi quota ketat
# dan akan memblokir IP jika melebihi batas.
def call_bank_gateway():
    pass
"#;

    let items = extractor.extract_from_code("payment_service", "gateway.py", code);
    assert_eq!(items.len(), 1);
    let item = &items[0];

    assert_eq!(item.knowledge_type, KnowledgeType::Decision);
    assert!(item.tags.contains(&"decision".to_string()));
    assert!(item.tags.contains(&"alasan".to_string()));
    assert!(item.content.contains("upstream API vendor bank memiliki limitasi"));
    assert_eq!(item.evidence[0].start_line, Some(2));
    assert_eq!(item.evidence[0].end_line, Some(4));
}

#[test]
fn test_block_comment_aggregation() {
    let extractor = KnowledgeExtractor::new();
    let code = r#"
/*
 * PERINGATAN: Thermal printer driver
 * Membutuhkan format 48 karakter per baris untuk kertas 80mm.
 * Jangan ubah padding font tanpa sinkronisasi print template.
 */
function printReceipt() {}
"#;

    let items = extractor.extract_from_code("pos_system", "printer.ts", code);
    assert_eq!(items.len(), 1);
    let item = &items[0];

    assert_eq!(item.knowledge_type, KnowledgeType::Warning);
    assert!(item.tags.contains(&"warning".to_string()));
    assert!(item.tags.contains(&"peringatan".to_string()));
    assert!(item.content.contains("Thermal printer driver"));
    assert!(item.content.contains("Membutuhkan format 48 karakter"));
    assert_eq!(item.evidence[0].start_line, Some(3));
    assert_eq!(item.evidence[0].end_line, Some(6));
}

#[test]
fn test_inline_comments_do_not_aggregate_code() {
    let extractor = KnowledgeExtractor::new();
    let code = r#"
let timeout = 5000; // @warning timeout terlalu kecil akan memicu false retry
let next_code = 123;
let final_code = 456;
"#;

    let items = extractor.extract_from_code("proj", "test.rs", code);
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item.evidence[0].start_line, Some(2));
    assert_eq!(item.evidence[0].end_line, Some(2));
    assert!(!item.content.contains("next_code"));
}

#[test]
fn test_bilingual_natural_tags() {
    let extractor = KnowledgeExtractor::new();
    let code = r#"
// BATASAN: Maximum 50 items per transaksi
// ATURAN: Diskon member hanya berlaku pada item non-promo
// CRITICAL: Database password tidak boleh di-hardcode
"#;

    let items = extractor.extract_from_code("proj", "rules.ts", code);
    assert_eq!(items.len(), 3);

    assert_eq!(items[0].knowledge_type, KnowledgeType::Constraint);
    assert!(items[0].tags.contains(&"constraint".to_string()));
    assert!(items[0].tags.contains(&"batasan".to_string()));

    assert_eq!(items[1].knowledge_type, KnowledgeType::BusinessRule);
    assert!(items[1].tags.contains(&"rule".to_string()));
    assert!(items[1].tags.contains(&"aturan".to_string()));

    assert_eq!(items[2].knowledge_type, KnowledgeType::Warning);
    assert!(items[2].tags.contains(&"critical".to_string()));
}

#[test]
fn test_markdown_bilingual_extraction() {
    let extractor = KnowledgeExtractor::new();
    let md = r#"
# Dokumentasi POS

## Arsitektur Sistem
Sistem menggunakan clean architecture dengan Tauri IPC bridge.

## Catatan Perbaikan & Bug Fix
Ditemukan memory leak pada listener barcode scanner.
Telah diperbaiki dengan unlisten on unmount.

## Batasan & Keamanan
Koneksi ke database local menggunakan encrypted SQLite cipher.
"#;

    let items = extractor.extract_from_markdown("proj", "README.md", md);
    assert_eq!(items.len(), 3);

    assert_eq!(items[0].knowledge_type, KnowledgeType::Architecture);
    assert_eq!(items[0].title, "Arsitektur Sistem");

    assert_eq!(items[1].knowledge_type, KnowledgeType::Warning);
    assert_eq!(items[1].title, "Catatan Perbaikan & Bug Fix");

    assert_eq!(items[2].knowledge_type, KnowledgeType::Constraint);
    assert_eq!(items[2].title, "Batasan & Keamanan");
}

**PRODUCT REQUIREMENTS DOCUMENT**  
**Engineering Memory & Codebase Intelligence System**  
| | |  
|-|-|  
| **Product Suite:** Engineering Memory & Codebase Intelligence | **Version / Status:** v0.2 — Comprehensive Architectural PRD |   
| **Core Pillars:** 1. Engineering Memory Layer  |  2. Canonical Code Logic Graph (CCLG) | **Target Audience:** Engineering Leadership, AI Engineers, Software Architects |   
| **Category:** Developer Infrastructure / Code Intelligence / Semantic Graph / Static Analysis | **Execution Model:** Local-First Engine with Git-Driven Incremental Indexing |   
| **Primary Interfaces:** CLI (`project`, `clg`) + Local Embedded Engine | **Integration Targets:** VS Code Extension + Model Context Protocol (MCP) + Web UI |   
   
**Executive Framing: Dua Fondasi yang Saling Mengunci**  
Dokumen ini mendefinisikan arsitektur lengkap platform Codebase Intelligence modern melalui dua pilar yang saling melengkapi secara fundamental:  
- PART I — Engineering Memory & Codebase Knowledge System: Lapisan kognitif proyek yang menghubungkan kode, komentar, dokumen, keputusan arsitektur, batasan (constraints), riwayat kegagalan, worklog, dan sesi AI. Lapisan ini menjawab pertanyaan mendasar: 'MENGAPA software bekerja seperti itu?' (Why and how it evolved).  
- PART II — Canonical Code Logic Graph (CCLG): Model graf standar perantara (canonical intermediate representation) yang memetakan sintaksis, simbol, call sites, data flow, control flow, exception flow, dependensi, dan interaksi sistem eksternal lintas bahasa. Lapisan ini menjawab pertanyaan teknis konkret: 'BAGAIMANA software bekerja?' (How software behaves).  
| |  
|-|  
| **■ Piramida Arsitektur Platform **┌────────────────────────────────────────────────────────┐│             PART I: ENGINEERING MEMORY                 ││  Decisions • Constraints • Business Rules • Worklog   ││  Architecture Drift • Technical Debt • AI Sessions    │└───────────────────────────┬────────────────────────────┘                            │ (Semantic Linking & Grounding)                            ▼┌────────────────────────────────────────────────────────┐│          PART II: CANONICAL CODE LOGIC GRAPH           ││  Symbols • CallSites • Control Flow • Data Flow        ││  Lineage • Type Graph • Impact Analysis • CFG Diff     │└───────────────────────────┬────────────────────────────┘                            │ (AST Parsing & Extraction)                            ▼┌────────────────────────────────────────────────────────┐│                      SOURCE CODE                       ││  TypeScript • Rust • Python • Go • Polyglot Repos      │└────────────────────────────────────────────────────────┘ |   
   
**PART I — CORE PILLAR**  
**ENGINEERING MEMORY & CODEBASE KNOWLEDGE SYSTEM**  
*Lapisan Kognitif, Konteks Arsitektur, Keputusan, dan Evolusi Codebase*  
**1. Executive Summary**  
Software development modern semakin bersifat inkremental, cepat, dan didorong oleh AI coding agent (paradigma vibe coding). Developer dapat membuat perubahan dalam volume besar hanya dalam hitungan menit. Namun, kapasitas repository untuk mengingat alasan, konteks konseptual, keputusan arsitektur, struktur tersembunyi, dan sejarah evolusinya tidak berkembang secepat pertumbuhan baris kode itu sendiri.  
Ekosistem rekayasa saat ini terfragmentasi ke berbagai pulau data:  
- Git menyimpan delta perubahan baris kode (diffs).  
- Dokumentasi menyimpan teks deskriptif yang rentan mengalami drift.  
- Issue tracker menyimpan pembagian tugas dan tiket pengerjaan.  
- Komentar menyimpan pengetahuan lokal yang terfragmentasi.  
- Architecture diagram menyimpan visi struktur awal yang kian tererosi.  
- AI conversation menyimpan konteks percakapan yang hilang saat sesi ditutup.  
- Worklog menyimpan catatan ad-hoc engineer.  
Produk ini membangun Engineering Memory Layer yang memetakan relasi antara Code, Comments, Documentation, Architecture, Decisions, Constraints, Dependencies, Worklog, Project State, Git History, dan AI Sessions ke dalam satu Codebase Knowledge Graph. Sistem tidak menggantikan Git, melainkan menjadi lapisan penghubung yang mengembalikan pemahaman holistik.  
**2. Problem Statement**  
**2.1 Knowledge Fragmentation**  
Pengetahuan teknis sebuah proyek tersebar tanpa indeks konseptual terpusat:  
| |  
|-|  
| **■ Peta Fragmentasi Pengetahuan **Source Code├── Comments├── README & Docs├── Git Commits & Diffs├── Issues & Pull Requests├── Architecture Diagrams├── TODOs & FIXMEs├── Worklog & Dev Notes└── AI Sessions / Prompts |   
   
Akibatnya, tidak ada satu sistem yang secara konsisten mampu menjawab: 'Apa yang sebenarnya perlu saya ketahui sebelum mengubah bagian ini?'  
**3. Core Problems**  
**3.1 Comment Discoverability**  
Komentar berharga di dalam kode sangat sulit ditemukan kembali. Developer mungkin ingat bahwa pernah ada alasan mengapa timeout diset 20 detik, namun tidak mengingat file, baris, keyword, atau wording pastinya. Pencarian teks biasa (grep) gagal karena developer mencari kata kunci yang berbeda dengan apa yang ditulis.  
**3.2 Comment Navigation**  
Komentar tersebar di ribuan file tanpa daftar isi konseptual (Conceptual Table of Contents):  
| |  
|-|  
| **■ Komentar Terisolasi Tanpa Struktur **file A └── comment (auth logic)file B └── comment (retry strategy)file C └── comment (database lock caveat) |   
   
Sistem harus menyatukan komentar-komentar terfragmentasi ke dalam taksonomi domain terstruktur: Authentication, Payment, Database, Security, Performance, Architecture, dan Business Rules.  
**4. Comment History Problem**  
Git mampu menampilkan perubahan baris komentar secara sintaksis, namun Git tidak memahami bagaimana sebuah pengetahuan (knowledge state) berevolusi. Perhatikan evolusi berikut:  
| |  
|-|  
| **■ Evolusi Knowledge State **2026-01: 'Redis digunakan untuk caching.'2026-03: 'Redis digunakan untuk caching dan distributed lock.'2026-05: 'Redis hanya digunakan untuk distributed lock.' |   
   
Yang bergeser bukan sekadar teks, melainkan asumsi arsitektur. Sistem mendeteksi siklus hidup pengetahuan: Created, Modified, Strengthened, Weakened, Superseded, Contradicted, Removed, dan Archived.  
**5. Documentation Drift**  
Dokumentasi statis kerap menyatakan alur Payment langsung ke PostgreSQL, padahal kode aktual telah memasukkan Redis dan Message Queue. README dapat menyatakan modul berada di 'src/services/', sementara refactoring telah memindahkannya ke 'src/modules/'.  
Sistem mendeteksi inkonsistensi multi-arah antara Documentation, Architecture, Code, dan Git History.  
**6. Architecture Drift**  
Arsitektur berlapis yang awalnya bersih perlahan mengalami boundary erosion seiring penambahan fitur cepat:  
| |  
|-|  
| **■ Visualisasi Architecture Drift **Arsitektur Awal (Clean Layers):Controller ───> Service ───> Repository ───> Database Arsitektur Aktual (Boundary Erosion):Controller ───> Service ───> Repository ───> DatabaseController ─────────────────────────────────> Database (Bypass Layer!)Service ──────> Controller (Reverse Dependency!)Utility ──────> Business Logic (Domain Leakage!) |   
   
Meskipun kode tetap berjalan, batas arsitektur telah rusak. Sistem mendeteksi dependency violation, circular dependency, unexpected coupling, layer violation, boundary erosion, dan domain leakage.  
**7. Folder Structure Drift**  
Struktur folder berkembang secara inkremental tanpa tata kelola semantik. Struktur domain yang rapi perlahan terpolusi oleh folder pembuangan darurat (dumping grounds) seperti common/, shared/, utils/, helpers/, misc/, dan temp/. Sistem menganalisis struktur semantik direktori, bukan sekadar filesystem fisik.  
**8. File Bloat**  
File bertumbuh dari 200 LOC menjadi 2.500+ LOC. Masalah sesungguhnya bukan sekadar jumlah baris, melainkan akumulasi tanggung jawab ganda, degradasi kohesi, serta lonjakan coupling.  
| |  
|-|  
| **■ Diagnosis File Bloat **payment/service.ts• LOC: 2,843• Responsibilities: 7 distinct domains• Incoming Dependencies: 14 modules• Complexity: High (Cyclomatic & Cognitive)• Growth Velocity: +1,490% across 103 commits• Actionable Recommendation: Split into PaymentProcessor, RefundHandler, and WebhookDispatcher. |   
   
**9. Modularity Drift**  
Sebuah modul awalnya memiliki satu tanggung jawab (Single Responsibility Principle), lalu perlahan ditambahkan fitur lain: UserService menelan email, billing, payment, CRM, dan notification. Sistem mendeteksi 'responsibility creep' ini melalui semantic clustering simbol.  
**10. DRY / Duplication Problem**  
Duplikasi modern tidak selalu berupa blok kode identik. Sistem mengklasifikasikan duplikasi ke dalam 3 tingkatan:  
- Level 1 — Textual Duplication: Kesamaan teks/token harfiah.  
- Level 2 — Structural Duplication: Kesamaan struktur AST.  
- Level 3 — Semantic Duplication: Fungsi berbeda nama (misal: calculateTotal(), calculateFinalAmount(), getPayableAmount()) namun secara konseptual melakukan pekerjaan yang sama.  
| |  
|-|  
| **■ Deteksi Semantic Duplication **Possible Semantic Duplication DetectedModules: payment/calculate.ts <───> billing/amount.tsSimilarity: 87%Shared Responsibility: Amount calculation with tax and discount adjustmentsRecommendation: Provide evidence and extract shared business logic component. |   
   
**11. Technical Debt Accumulation**  
Technical debt terakumulasi perlahan: Commit 1 (small shortcut) → Commit 12 (workaround) → Commit 37 (duplicate logic) → Commit 61 (new dependency) → Commit 103 (architecture violation). Sistem menyusun Technical Debt Timeline untuk memperlihatkan bagaimana hutang terbentuk.  
**12. Lost Decisions**  
Developer kerap lupa alasan di balik pemilihan teknologi dan konfigurasi: Mengapa Postgres? Mengapa Redis? Mengapa webhook? Mengapa timeout 20 detik? Sistem menghubungkan keputusan tersebut langsung dengan implementasi kodenya.  
**13. Lost Failed Attempts**  
Git menyimpan bukti bahwa sebuah pendekatan pernah dicoba lalu dihapus, tetapi tidak menjelaskan MENGAPA pendekatan tersebut gagal. Tanpa memori kegagalan, developer baru atau AI agent akan mengulangi kesalahan eksperimen yang sama.  
| |  
|-|  
| **■ Pencatatan Failed Attempt **Failed Approach Record:• Approach: Redis-based distributed locking• Reason Rejected: Race condition under multi-region active-active deployment• Replaced By: PostgreSQL transaction advisory lock (src/lock/pg_advisory.ts)• Provenance: PR #142 incident resolution |   
   
**14. Lost Project State**  
Setelah maraton sesi AI coding, developer kerap bingung: Apa yang sudah selesai? Apa yang belum? Apa yang berubah? Apa yang rusak? Sistem menyediakan Project Snapshot untuk menangkap kondisi riil proyek terkini.  
**15. AI Context Fragmentation**  
Vibe coding melahirkan pola terfragmentasi lintas sesi AI di mana konteks setiap sesi hilang saat sesi ditutup. Produk menyediakan Persistent AI Engineering Memory.  
**16. Product Vision**  
| |  
|-|  
| **■ Visi Produk Engineering Memory **Make every codebase remember how and why it evolved. |   
   
Sistem menjadi lapisan memori kognitif antara Developer, AI Agent, Engineering Memory, Codebase, dan Project State.  
**17. Product Principles**  
**17.1 Git Remains the Source of Truth**  
Produk tidak menggantikan Git. Git tetap menjadi fondasi sumber kode, commit, branch, diff, dan riwayat. Sistem membangun knowledge layer di atas Git.  
**17.2 Evidence First**  
Setiap item pengetahuan wajib memiliki bukti asal-usul (provenance) berupa file, baris, commit, PR, atau komentar.  
**17.3 No Mandatory Documentation Burden**  
Developer tidak dipaksa mengisi form metadata panjang. Komentar alami sederhana sudah cukup bagi engine untuk diekstraksi secara semantik.  
**17.4 Human-Readable & Zero Lock-In**  
Format data terbuka dan transparan. Jika engine dinonaktifkan, repository tetap normal tanpa ketergantungan proprietary.  
**18. Core Product: Knowledge Engine**  
Knowledge Engine mengumpulkan Source Code, Comments, Docs, Git, Architecture, Issues, PRs, dan AI Sessions, lalu menghasilkan Knowledge Graph, Code Graph, dan History Graph terintegrasi.  
**19. Knowledge Graph Schema**  
- Nodes: Project, Domain, Module, File, Symbol, Comment, Decision, Constraint, Warning, Business Rule, Architecture, Dependency, Worklog, Issue, Commit, AI Session.  
- Relationships: affects, implements, depends_on, related_to, introduced_by, changed_by, supersedes, contradicts, documents, constrains, belongs_to, caused_by, replaced_by.  
**20. Code Graph**  
Memetakan hierarki struktural konkret: Project → Directory → Module → File → Symbol → Function → Dependency → Call Graph.  
**21. Knowledge Index (TOC)**  
Sistem menyusun daftar isi konseptual: 01 Architecture, 02 Decisions, 03 Constraints, 04 Business Rules, 05 Warnings, 06 Worklog, 07 History.  
**22. Knowledge Network Diagram**  
Memungkinkan traversal graf multi-arah: Comment → Decision → File → Module → Architecture → Dependencies → History, atau Decision → Affected Code → Commits → Tests → Documentation.  
**23. Semantic Comment Search**  
Pencarian tidak bergantung pada exact wording. Pertanyaan 'Kenapa timeout payment 20 detik?' menemukan komentar di 'src/payment/client.ts:87' yang menjelaskan latensi downstream provider.  
**24. Context Retrieval for AI**  
AI tidak perlu membaca seluruh repository. Sistem menyajikan 'Minimal Context Package' relevan dengan prinsip: Maximum relevance with minimum tokens.  
**25. Knowledge Timeline**  
Setiap item pengetahuan memiliki riwayat status: Created, Modified, Constraint Added, Implementation Changed, hingga Superseded.  
**26. Knowledge Diff**  
Berbeda dengan Git diff (sintaksis teks), Knowledge Diff mencatat pergeseran makna: 'CONSTRAINT CHANGED: Payment timeout dari 10s ke 20s karena latensi provider meningkat; Berdampak pada PaymentClient, RetryManager, dan Checkout flow.'  
**27. Architecture Health**  
Engine menghitung skor kesehatan berbasis bukti (evidence-backed): Modularity (72), Cohesion (68), Coupling (51), Duplication (39), Structure (74), Boundary (61), Technical Debt (48).  
**28. Refactoring Radar**  
- **[CRITICAL] **Critical: payment/service.ts (2,843 LOC, 7 responsibilities).  
- **[HIGH] **High: auth ──> billing (Circular dependency violation).  
- **[HIGH] **High: utils/ (23 modul tanpa kohesi fungsional).  
- **[MEDIUM] **Medium: Duplikasi validasi logika pembayaran.  
- **[MEDIUM] **Medium: Komentar usang bertentangan dengan implementasi runtime.  
**29. Structural Evolution**  
Merekam evolusi modularitas antar-rilis: v0.1 (12 modul) → v0.2 (17 modul) → v0.3 (24 modul) → v0.4 (31 modul, 8 large files, 14 boundary violations).  
**30. Project Snapshot**  
Menghasilkan ringkasan status instan: Goal, Completed items, Architecture changes, New decisions, Active constraints, Warnings, Files changed, dan Next steps.  
**31. Development Worklog**  
Setiap sesi coding menghasilkan catatan terstruktur: Goal, Tindakan teknis, Keputusan yang diambil, File disentuh, Status pengujian, dan Isu tertunda.  
**32. AI Session Memory**  
Menghubungkan rantai kausalitas: AI Session → Prompt → Keputusan → Perubahan Kode → Commit → Test → Knowledge.  
**33. AI / MCP Interface Specification**  
Mengekspos kemampuan kognitif melalui Model Context Protocol (MCP): knowledge.search, knowledge.related, knowledge.decisions, knowledge.constraints, knowledge.history, project.snapshot, project.architecture, project.health, code.dependencies, code.impact, refactor.analyze.  
**34. Command-Line Interface (CLI)**  
| |  
|-|  
| **■ CLI Syntax Overview **$ project init                           # Inisialisasi engine pada repo$ project map                            # Memetakan struktur & relasi repo$ project knowledge                      # Menampilkan Conceptual TOC$ project search "payment timeout"       # Semantic search$ project related src/payment/service.ts # Menampilkan keterkaitan simbol$ project ask "why does payment use redis?" # Natural language query$ project context "refactor retry"       # Konteks minimal untuk AI$ project history src/payment/service.ts # Linimasa evolusi modul$ project knowledge-diff HEAD~10..HEAD   # Diff tingkat pengetahuan$ project architecture                   # Evaluasi integritas arsitektur$ project health                         # Skor kesehatan codebase$ project refactor src/payment/service.ts# Rekomendasi refactoring$ project snapshot                       # Ringkasan status terkini$ project timeline                       # Linimasa technical debt |   
   
**35. Initial MVP Scope**  
- Repository indexing: Git, source code, comments, Markdown documentation.  
- Knowledge extraction: decisions, constraints, warnings, TODO, business rules.  
- Knowledge Index: generasi Conceptual Table of Contents.  
- Semantic Search: pencarian alasan di balik keputusan teknis.  
- Knowledge Graph: Comment ──> File ──> Symbol ──> Module ──> Commit.  
- CLI: project init, map, search, related, context, history.  
**36. MVP Non-Goals**  
Bukan bagian dari MVP: Automatic code refactoring, automatic file moving, architecture rewriting sepihak, pengganti issue tracker, pengganti Git, autonomous coding agent, cloud backend. Prinsip MVP: Observe and explain before modify.  
**37. Proposed System Architecture**  
| |  
|-|  
| **■ Arsitektur Sistem Engineering Memory **┌─────────────────────────┐                    │     Developer / AI      │                    └────────────┬────────────┘                                 │ CLI / MCP / IDE                                 ▼                    ┌─────────────────────────┐                    │    Knowledge Engine     │                    └────────────┬────────────┘                                 │           ┌─────────────────────┼─────────────────────┐           ▼                     ▼                     ▼    ┌─────────────┐       ┌─────────────┐       ┌─────────────┐    │    Code     │       │  Knowledge  │       │   History   │    │  Analyzer   │       │  Extractor  │       │  Analyzer   │    └──────┬──────┘       └──────┬──────┘       └──────┬──────┘           │                     │                     │           └─────────────────────┼─────────────────────┘                                 ▼                    ┌─────────────────────────┐                    │     Knowledge Graph     │                    └────────────┬────────────┘                                 │           ┌─────────────────────┼─────────────────────┐           ▼                     ▼                     ▼     Search Index           Graph Store            Timeline           │                     │                     │           └─────────────────────┼─────────────────────┘                                 ▼                     Context Retrieval Pipeline                                 │                                 ▼                           AI / Developer |   
   
**38. Data Model Specification**  
| |  
|-|  
| **■ Contoh Knowledge Node (JSON) **{  "id": "knowledge_001",  "type": "decision",  "title": "Webhook is payment source of truth",  "content": "Payment status must be determined from webhook events.",  "status": "active",  "confidence": 0.94,  "evidence": [    { "type": "comment", "file": "src/payment/webhook.ts", "line": 42 },    { "type": "commit", "id": "8a31f" }  ],  "affects": ["PaymentService", "Checkout", "Subscription"]} |   
   
**39. Provenance & Confidence Model**  
Sumber bukti (human, comment, documentation, commit, issue, PR, AI, inferred) dinilai dengan tingkat keyakinan (verified, high, medium, low, unknown) melalui cross-validation multi-sumber.  
**40. Contradiction Detection**  
Mendeteksi pertentangan logis antara dokumen rancangan dan kode runtime (contoh: Dokumen menyebut Redis untuk caching, sementara kode menggunakannya untuk distributed lock).  
**41. Knowledge Regression**  
Analog dengan regression test: mendeteksi hilangnya batasan/invarian kritis (misal: 'Payment must be idempotent') pada commit baru sebelum masuk ke produksi.  
**42. Impact Analysis**  
Sebelum perubahan dieksekusi, sistem mengevaluasi dampak langsung, dependensi turunan, aturan bisnis terkait, cakupan pengujian, dan level risiko modifikasi.  
**43. Success Metrics**  
- Penurunan waktu orientasi modul baru hingga >50%.  
- 30% – 70% reduksi token konteks yang tidak relevan yang dikirimkan ke model AI.  
- Latensi respon penelusuran pengetahuan repository di bawah 5 detik (< 5s).  
- Penurunan rollback rate terhadap kode yang dihasilkan AI coding agent.  
**44. Product Differentiation**  
| | |  
|-|-|  
| **Kategori Tool** | **Fokus Utama & Batasan** |   
| **Git** | What changed — Berfokus pada delta baris kode, hash commit, branch, diff, dan author timestamp. |   
| **Documentation** | What is known — Menyimpan teks statis deskriptif yang rentan mengalami drift dan usang. |   
| **Issue Tracker** | What should be done — Melacak alokasi tiket, backlog, sprint, dan penugasan tim. |   
| **Static Analyzer** | What may be wrong — Mendeteksi syntax error, type mismatches, security vulnerability, dan linting. |   
| **AI Coding Agent** | What can be changed — Menulis, memodifikasi, dan men-generate baris kode secara generatif. |   
| **Engineering Memory** | Why, how, where, & how it evolved — Mengintegrasikan konteks keputusan, batasan, struktur, dan evolusi ke dalam satu graf. |   
   
**45. Killer Use Cases**  
- **Use Case 1 — **Developer Onboarding: Menjalankan `project map` untuk memahami arsitektur, batasan, dan keputusan historis tanpa membaca puluhan ribu baris kode mentah.  
- **Use Case 2 — **Investigasi 'Kenapa Angka Ini Begini?': Menjawab alasan spesifik di balik konstanta konfigurasi tanpa perlu exact matching pada teks komentar.  
- **Use Case 3 — **Persiapan Konteks AI Agent: Menyediakan paket konteks yang presisi bagi AI coding agent agar tidak melanggar batasan arsitektur aktif.  
- **Use Case 4 — **Audit File Raksasa (File Bloat 3.000 LOC): Mengungkap riwayat pembengkakan file dan akumulasi tanggung jawab ganda guna memandu refactor.  
- **Use Case 5 — **Project Handoff yang Mulus: Mengeksekusi `project snapshot` untuk menghasilkan laporan kondisi sistem aktual saat pergantian engineer.  
**46. Long-Term Product Roadmap**  
| |  
|-|  
| **■ Roadmap Implementasi Multi-Fase **Fase 1: Knowledge Index, Semantic Search, Core Graph Engine, Git History.Fase 2: Architecture Health, Refactoring Radar, Knowledge Regression, Evolution Timeline.Fase 3: VS Code Native Extension, MCP Server, Persistent AI Session Memory, Context Optimization.Fase 4: Multi-Repo & Team Knowledge, PR Review Intelligence, Architecture Governance. |   
   
**47. Ultimate Product Model**  
| |  
|-|  
| **■ Ultimate Conceptual Framework **PROJECT                                 │           ┌─────────────────────┼─────────────────────┐           ▼                     ▼                     ▼         CODE                KNOWLEDGE              HISTORY           │                     │                     │           ▼                     ▼                     ▼     • Structure           • Decisions           • Changes     • Modules             • Constraints         • Timeline     • Symbols             • Warnings            • Evolution     • Dependencies        • Business Rules      • AI Sessions           │                     │                     │           └─────────────────────┼─────────────────────┘                                 ▼                          KNOWLEDGE GRAPH                                 │           ┌─────────────────────┼─────────────────────┐           ▼                     ▼                     ▼        Search                Context               Health           │                     │                     │           └─────────────────────┼─────────────────────┘                                 ▼                           Developer / AI |   
   
**48. Product Philosophy**  
Codebase bukan hanya kumpulan kode. Ia adalah rekaman formal keputusan manusia. Setiap folder menunjukkan keputusan arsitektur; setiap abstraksi menunjukkan desain; setiap workaround menunjukkan sejarah masalah; setiap komentar menyimpan pengetahuan; setiap commit mencatat arah perubahan; setiap sesi AI mencatat proses penalaran. Tugas Engineering Memory adalah menghubungkan kembali semuanya.  
**49. Core Product Statement**  
| |  
|-|  
| **■ Core Product Positioning **Git remembers what changed.Engineering Memory remembers why. Positioning Teknis:A knowledge graph for codebase structure, decisions, constraints, and evolution. Value Proposition AI:Give coding agents project memory, not your entire repository. |   
   
**50. Final Product Definition & Strategic Synthesis**  
Engineering Memory adalah sistem intelligence layer untuk repository yang secara otonom membangun peta hubungan antara kode, komentar, dokumentasi, arsitektur, keputusan, constraint, dependency, worklog, AI session, dan Git history. Tujuannya adalah mempertahankan pemahaman (comprehension retention) terhadap software ketika software tersebut terus berubah dengan cepat.  
**PART II — CORE PILLAR**  
**CANONICAL CODE LOGIC GRAPH (CCLG)**  
*Model Graf Standar Perantara Lintas Bahasa untuk Struktur, Eksekusi, dan Logika Kode*  
**1. Executive Summary (CCLG)**  
Canonical Code Logic Graph (CCLG) adalah model graph standar untuk merepresentasikan source code sebagai struktur yang dapat dipahami, ditelusuri, dibandingkan, dan dianalisis oleh manusia maupun AI. CCLG tidak berhenti pada hubungan sederhana antar file (file A ──> file B), melainkan memetakan hirarki logika komprehensif:  
| |  
|-|  
| **■ Hierarki Abstraksi CCLG **Project  ──>  Module  ──>  File  ──>  Class  ──>  Function  ──>  Execution   ──>  Condition  ──>  Data Flow  ──>  Function Call  ──>  External System |   
   
Dengan demikian sistem dapat memahami bukan hanya lokasi kode, melainkan: apa yang didefinisikan, siapa memanggil siapa, bagaimana data mengalir, bagaimana control flow bercabang, kondisi apa yang mengubah perilaku, dependensi yang digunakan, exception/error path yang tersedia, API & database yang disentuh, aturan bisnis yang terkandung, serta bagaimana seluruh struktur tersebut berevolusi sepanjang riwayat Git. CCLG menjadi canonical intermediate representation antara berbagai bahasa pemrograman dan seluruh fitur Code Intelligence.  
**2. Problem Statement: Keterbatasan File Dependency Graph**  
Codebase modern memiliki masalah besar: struktur direktori file mudah dilihat, tetapi logika internal dan perilaku dinamis antar-kode sangat sulit dipetakan. File dependency graph konvensional hanya menunjukkan relasi file kasar:  
| |  
|-|  
| **■ File Graph Konvensional (Terlalu Dangkal) **controller.ts  ───>  service.ts  ───>  repository.ts |   
   
Padahal perilaku eksekusi nyata melibatkan aliran multi-fungsi dan mutasi data:  
| |  
|-|  
| **■ Alur Logika dan Aliran Data Nyata **POST /checkout   ↓CheckoutController.checkout()   ↓CheckoutService.checkout()   ↓PaymentService.createPayment()   ↓Order.total   ↓calculateDiscount()   ↓finalAmount   ↓PaymentGateway.charge()   ↓Stripe API   ↓PaymentRepository.save()   ↓Database.payments |   
   
Jika seorang developer mengubah fungsi `calculateDiscount()`, pertanyaan sesungguhnya adalah: Kode apa yang terdampak? Data apa yang berubah? API mana yang menerima data tersebut? Aturan bisnis apa yang ikut terpengaruh? Jalur error apa yang berubah? File dependency graph tidak mampu menjawab pertanyaan-pertanyaan ini.  
**3. Product Vision (CCLG)**  
| |  
|-|  
| **■ Visi Inti CCLG **Build a canonical semantic map of how software is structured, connected, and behaves. |   
   
Dalam bahasa sederhana: Mengubah source code menjadi peta logika yang dapat ditelusuri. CCLG menjadi fondasi teknis yang memungkinkan AI dan developer melihat codebase bukan sebagai kumpulan file teks, melainkan sebagai sistem logika yang saling terhubung.  
**4. Goals & Scope**  
**4.1 Primary Goals**  
CCLG dirancang untuk memenuhi 15 kapabilitas utama:  
- 1. Merepresentasikan struktur source code secara hierarkis.  
- 2. Merepresentasikan symbol dan deklarasi entitas.  
- 3. Merepresentasikan function call pada tingkat call site.  
- 4. Merepresentasikan control flow (CFG: branches, loops, jumps).  
- 5. Merepresentasikan data flow (derivation, transformation, sinks).  
- 6. Merepresentasikan dependency eksplisit maupun implisit.  
- 7. Merepresentasikan type relationships (inheritance, implements, generics).  
- 8. Merepresentasikan exception/error propagation flow.  
- 9. Merepresentasikan interaksi sistem eksternal (API, Database, Queue).  
- 10. Menyediakan pemetaan lokasi sumber (precise source mapping).  
- 11. Menyediakan rantai bukti asal-usul (provenance).  
- 12. Mendukung scoring tingkat keyakinan (confidence levels).  
- 13. Mendukung temporal graph yang terikat dengan snapshot commit Git.  
- 14. Mendukung graph diff untuk analisis perubahan perilaku (behavioral diff).  
- 15. Menjadi canonical model netral lintas berbagai bahasa pemrograman.  
**5. Non-Goals**  
CCLG bukan merupakan: compiler, debugger, IDE, pengganti Git, issue tracker, project management tool, automatic refactoring engine, autonomous coding agent, atau runtime APM monitoring system penuh. CCLG adalah representasi graf semantik yang dapat dikonsumsi oleh tool-tool tersebut.  
**6. Core Principles**  
**6.1 Code Is Evidence**  
Graph harus dibangun berdasarkan bukti konkret source code yang dapat diverifikasi, bukan berdasarkan tebakan bebas AI.  
**6.2 Relationships Must Be Typed**  
Dilarang menggunakan relasi generik `A ──depends_on──> B` jika relasi spesifiknya adalah `A ──calls──> B`, `A ──reads──> B`, atau `A ──mutates──> B`.  
**6.3 Source Location Is Not Identity**  
Perubahan nomor baris (line number) akibat penambahan baris baru di atasnya tidak boleh mengubah identitas unik dari simbol.  
**6.4 Provenance Is Mandatory**  
Setiap relasi penting wajib mampu menjawab 4 pertanyaan: Dari mana asalnya (Where)? Bagaimana dideteksi (How)? Seberapa yakin sistem (Certainty)? Kapan valid (When)?  
**6.5 Static Fact ≠ Inference**  
Sistem membedakan secara tegas antara fakta statis yang dapat dibuktikan secara deterministik dari AST, dan hasil inferensi semantik probabel.  
**7. Canonical Graph Formal Model**  
| |  
|-|  
| **■ Formal Model CCLG **G = (V, E, S, P) |   
   
Di mana V = Nodes (Simpul), E = Edges (Busur relasi berbobot & bertipe), S = Temporal Snapshots (Keadaan graf terikat commit Git), dan P = Provenance (Rantai bukti asal-usul). Graph harus dapat direkonstruksi secara deterministik pada snapshot commit tertentu.  
**8. Node Schema Specification**  
| |  
|-|  
| **■ Canonical Node Schema (JSON) **{  "id": "node:01HX8A...",  "kind": "function",  "name": "createPayment",  "project_id": "project:payment-core",  "language": "typescript",  "source": {    "file": "src/payment/service.ts",    "start_line": 42,    "start_column": 1,    "end_line": 87,    "end_column": 2  },  "identity": {    "qualified_name": "PaymentService.createPayment",    "signature": "createPayment(orderId: string): Promise<Payment>"  },  "state": { "status": "active" },  "provenance": { "source": "parser", "confidence": 1.0 }} |   
   
**9. Node Taxonomy**  
Taxonomy simpul dibagi ke dalam 7 kelompok entitas:  
- 9.1 Structural Nodes: project, package, module, directory, file, namespace.  
- 9.2 Symbol Nodes: class, interface, struct, enum, function, method, constructor, variable, parameter, property, constant, type, generic.  
- 9.3 Execution Nodes: entry, exit, statement, expression, condition, branch, loop, return, throw, catch, finally, call_site.  
- 9.4 Data Nodes: value, object, collection, field, database, table, column, query.  
- 9.5 External Nodes: api, endpoint, service, queue, topic, filesystem, process.  
- 9.6 Knowledge Nodes: comment, decision, constraint, business_rule, warning, assumption.  
- 9.7 History Nodes: commit, branch, tag, change, snapshot.  
**10. Node Identity & Stability**  
Identitas simpul harus stabil terhadap pergeseran baris. Formula identitas canonical:  
| |  
|-|  
| **■ Formula Identitas Simpul Stabil **language + namespace/module + qualified_name + symbol_kindContoh: symbol:typescript:payment.PaymentService.createPayment |   
   
Line number hanya bertindak sebagai source locator, bukan pengidentifikasi unik.  
**11. Execution Identity: Symbol vs. CallSite**  
Simbol fungsi abstrak harus dibedakan dari lokasi pemanggilan konkretnya (CallSite). Pola relasi:  
| |  
|-|  
| **■ Pola Pemisahan Simbol dan CallSite **Function  ───contains───>  CallSite  ───resolves_to───>  Function |   
   
Hal ini memungkinkan sistem membedakan pemanggilan berulang ke fungsi yang sama di dalam satu blok fungsi dengan argumen berbeda.  
**12. Edge Schema Specification**  
| |  
|-|  
| **■ Canonical Edge Schema (JSON) **{  "id": "edge:01HX9B...",  "from": "node:call_site_61",  "to": "node:function_charge",  "kind": "calls",  "qualifier": { "argument_index": 0, "parameter": "amount" },  "provenance": {    "source": "ast",    "confidence": 0.99,    "evidence": [{ "file": "src/payment/service.ts", "start_line": 61, "end_line": 61 }]  },  "temporal": {    "valid_from": "snapshot:commit_abc123",    "valid_until": null  }} |   
   
**13. Canonical Edge Taxonomy**  
Busur relasi dikelompokkan ke dalam 8 kategori semantik:  
- 13.1 Structural: contains, declares, defines, exports, imports, belongs_to.  
- 13.2 Symbol: references, uses, extends, implements, instantiates, overrides, aliases, resolves_to.  
- 13.3 Behavioral: calls, returns, passes, reads, writes, assigns, mutates, creates, destroys, throws, catches, handles.  
- 13.4 Control Flow: flows_to, branches_to, loops_to.  
- 13.5 Data Flow: derives_from, flows_to, transforms, filters, maps, aggregates.  
- 13.6 External: calls_api, reads_database, writes_database, publishes, subscribes, sends, receives, reads_file, writes_file, spawns.  
- 13.7 Knowledge: documented_by, constrained_by, implements_rule.  
- 13.8 History: introduced_by, modified_by, supersedes, contradicts, derived_from.  
**14. Relationship Semantics**  
Setiap jenis relasi memiliki semantik ketat dan tidak boleh dicampuradukkan:  
- Call: `A ──calls──> B` (A mengaktifkan eksekusi subrutin B).  
- Data Flow: `A ──flows_to──> B` (Data/nilai mengalir dari variabel A menuju konsumen B).  
- Control Flow: `Condition ──branches_to──> B` (Alur eksekusi bergantung pada evaluasi kondisi).  
- Data Derivation: `A ──derives_from──> B` (Nilai A dihitung atau ditransformasi dari sumber B).  
**15. Edge Qualifier**  
Qualifier menyematkan metadata kontekstual pada edge tanpa perlu menciptakan tipe edge baru untuk setiap variasi:  
| |  
|-|  
| **■ Contoh Edge Qualifier **Contoh Branch:{ "from": "condition:payment.success", "to": "function:save", "kind": "branches_to", "qualifier": { "condition": "true" } } Contoh Argument Passing:{ "from": "variable:finalAmount", "to": "call:charge", "kind": "flows_to", "qualifier": { "argument_index": 0, "parameter": "amount" } } |   
   
**16. Control Flow Model**  
Control Flow Graph (CFG) canonical memetakan percabangan eksekusi secara eksplisit:  
| |  
|-|  
| **■ Pola Percabangan Canonical **condition ├── branches_to {condition=true}  ──> save() └── branches_to {condition=false} ──> rollback() |   
   
CFG mencakup: sequential flow, conditional branch, loops, break, continue, return, exception throws, dan finally blocks.  
**17. Data Flow Model**  
Melacak pergerakan dan mutasi data:  
| |  
|-|  
| **■ Pemetaan Aliran Data (Data Flow Graph) **cart.total   ──derives_from──> cartdiscount     ──derives_from──> calculateDiscount()finalAmount  ──derives_from──> cart.totalfinalAmount  ──derives_from──> discountfinalAmount  ──flows_to─────> charge() |   
   
**18. Call Graph Model Berbasis CallSite**  
| |  
|-|  
| **■ Call Graph Berbasis CallSite **checkout()   │   └── contains ──> CallSite                      │                      ├── calls ──> createPayment()                      └── argument[0] <── flows_to <── orderId |   
   
**19. Exception Flow Model**  
Menganalisis bagaimana error disebarkan dan ditangani:  
| |  
|-|  
| **■ Alur Exception Propagation **charge() ├── throws ──> TimeoutError ─────catches───> retry() └── throws ──> PaymentDeclined ──catches───> cancelOrder() |   
   
**20. Type Graph Model**  
Menghubungkan relasi tipe data statis: `StripeGateway ──implements──> PaymentGateway`. Fungsi `charge()` mendeklarasikan `accepts: Payment` dan `returns: PaymentResult`.  
**21. External System Graph**  
Memetakan batasan sistem dengan dunia luar: `createPayment()` memiliki relasi `calls_api: Stripe`, `reads_database: orders`, dan `writes_database: payments`.  
**22. Logic Graph & Boolean Modeling**  
Merepresentasikan aturan evaluasi kondisional kompleks:  
| |  
|-|  
| **■ Logic Graph Boolean Composition **user.is_verified ──┐                   ├── AND ──> require_manual_review()order.total > 1M ──┘ |   
   
Mendukung operator: AND, OR, NOT, ==, !=, >, <, >=, <=, IN, NOT_IN, EXISTS, NULL_CHECK, dan STATE_CHECK.  
**23. Business Rule Extraction & Confidence**  
Pola kode seperti `if user.is_admin` atau `if amount > 1_000_000` diidentifikasi sebagai potensi aturan otorisasi atau threshold. Hasil analisis otomatis ini diberi tag: `source=semantic_analysis`, `certainty=inferred`, `confidence=0.85`.  
**24. Provenance Tracking**  
Setiap simpul dan relasi mencatat sumber deteksinya: parser, ast, type_checker, symbol_resolver, cfg_analysis, dataflow_analysis, sql_analysis, git, runtime_trace, human, atau ai_inference beserta range baris bukti.  
**25. Certainty Model**  
- **CERTAIN — **Certain: Terbukti secara deterministik melalui AST direct call.  
- **LIKELY — **Likely: Resolusi antarmuka polimorfik atau dynamic dispatch dengan kandidat terbatas.  
- **INFERRED — **Inferred: Resolusi berbasis refleksi runtime atau prediksi model semantik.  
- **UNKNOWN — **Unknown: Perilaku yang sangat bergantung pada konfigurasi dinamis eksternal.  
**26. Temporal Graph & Snapshotting**  
Relasi menyimpan masa berlaku: `valid_from: snapshot_A`, `valid_until: snapshot_B`. Memungkinkan rekonstruksi graf secara akurat pada sembarang commit masa lalu.  
**27. Behavioral Diff vs. Line Diff**  
CCLG membandingkan perbedaan perilaku sistem antar-commit:  
| |  
|-|  
| **■ Contoh Behavioral Diff **BEHAVIORAL CHANGE DETECTED:Before: checkout ──> charge ──> saveAfter:  checkout ──> fraud_check ──> charge ──> retry ──> saveModifications:[+] Added Call Path: fraud_check()[+] Added Exception Handler: retry on TimeoutError[*] Changed Data Flow: fraud_score injected into charge parameters |   
   
**28. Impact Analysis Engine**  
| |  
|-|  
| **■ Laporan Impact Analysis **$ clg impact PaymentGateway.chargeDIRECT DEPENDENTS:  • PaymentService.createPayment()  • PaymentService.retryPayment()INDIRECT CALLERS (Depth 2+):  • CheckoutService.checkout()  • SubscriptionService.renew()EXPOSED APIS:  • POST /checkout  • POST /subscription/renewAFFECTED DATA STORES:  • payments, ordersCONSTRAINED BY:  • Rule: Payment operations must be idempotent |   
   
**29. Reverse Analysis (Callers & Usages)**  
Melakukan penelusuran hulu: `PaymentGateway.charge <── PaymentService <── CheckoutService <── POST /checkout`.  
**30. End-to-End Path Analysis**  
Mencari lintasan lengkap dari entry point hingga storage: API Endpoint ──> Controller ──> Service ──> Gateway ──> Database.  
**31. Data Lineage (Source to Sink)**  
Menjawab asal muasal nilai (`where does payment.amount come from?`) dan ke mana nilai tersebut diteruskan (`where is cart.total used?`).  
**32. Architecture Rules Enforcement**  
Mendeteksi pelanggaran aturan struktural. Contoh: jika Controller memiliki relasi langsung `reads_database` ke Database, sistem langsung membunyikan alarm `ARCHITECTURE VIOLATION: Layer Bypassed`.  
**33. Semantic Duplication Detection**  
Mendeteksi kesamaan pola graf alur data meskipun nama variabel dan fungsi berbeda 100% (misal pola: `total ──> discount ──> tax ──> finalAmount`).  
**34. Dead Logic Detection**  
Simpul yang memiliki incoming calls = 0, exports = 0, dan references = 0 ditandai sebagai `POSSIBLE DEAD LOGIC`.  
**35. Graph-Based Complexity Metrics**  
Mengukur kompleksitas berbasis graf nyata: jumlah nodes (48), branches (12), outgoing calls (17), cycles (2), external systems (4), dan data transformations (11).  
**36. Language Adapter Architecture**  
| |  
|-|  
| **■ Arsitektur Language Adapter **CANONICAL GRAPH (CCLG)                                   ▲             ┌─────────────────────┼─────────────────────┐             │                     │                     │     TypeScript Adapter      Python Adapter         Rust Adapter             │                     │                     │       tree-sitter / TS        ast / LibCST             syn / ra |   
   
**37. Incremental Analysis Pipeline**  
Perubahan pada satu file hanya memicu parsing ulang simbol terkait, mengompilasi ulang subgraf lokal, dan mempropagasi dampak ke snapshot terkini tanpa memindai seluruh repository.  
**38. Storage Model Specification**  
Database relasional lokal (SQLite) dengan tabel inti: `projects`, `nodes`, `edges`, `snapshots`, `evidence`, `node_snapshots`, dan `edge_snapshots`.  
**39. Query Engine CLI Syntax (`clg`)**  
| |  
|-|  
| **■ Perintah Query Engine CLI **$ clg calls <FUNCTION>             # Fungsi yang dipanggil target$ clg callers <FUNCTION>           # Fungsi yang memanggil target$ clg impact <SYMBOL>              # Analisis dampak perubahan simbol$ clg path <SOURCE> <TARGET>       # Jalur eksekusi antara dua titik$ clg dataflow <VARIABLE>          # Alur pergerakan variabel$ clg lineage <VARIABLE>           # Silsilah pembentukan nilai$ clg dependencies <SYMBOL>        # Dependensi langsung & transitif$ clg architecture                 # Verifikasi aturan arsitektur$ clg diff <COMMIT_A> <COMMIT_B>   # Perbandingan graf perilaku |   
   
**40. Human Visualization Interfaces**  
Menyajikan pohon eksekusi bersih pada CLI serta visualisasi node interaktif pada Web UI/VS Code.  
**41. AI Integration & Context Compression**  
| |  
|-|  
| **■ Pipeline Kompresi Konteks AI **User Intent  ──>  Semantic Search  ──>  Graph Traversal  ──>  Relevant Subgraph  ──>  Context Compression  ──>  Minimal Context Package  ──>  AI Agent |   
   
**42. Model Context Protocol (MCP) Tools**  
Mengekspos fungsi graf sebagai tools siap pakai untuk LLM Agent: `get_symbol`, `get_callers`, `get_callees`, `get_dataflow`, `get_impact`, `get_path`, `get_behavior_diff`, dan `get_context`.  
**43. Canonical Graph Invariants**  
- Invariant 1: Setiap node memiliki identitas stabil berbasis qualified name.  
- Invariant 2: Setiap edge memiliki source dan target node yang valid di dalam graf.  
- Invariant 3: Setiap edge wajib memiliki hubungan bertipe eksplisit (strictly typed).  
- Invariant 4: Setiap edge analitis wajib memiliki bukti asal-usul (provenance).  
- Invariant 5: Pergeseran posisi baris kode tidak boleh mengubah identitas simbol.  
- Invariant 6: Graph dapat direkonstruksi secara deterministik pada Git snapshot.  
- Invariant 7: Fakta statis deterministik dilarang disamarkan sebagai inferensi probabilistik.  
- Invariant 8: Call graph, control flow, dan data flow dilarang disatukan tanpa pemisahan semantik.  
- Invariant 9: Relasi generik `depends_on` dilarang menggantikan relasi yang lebih spesifik.  
- Invariant 10: Language adapter hanya bertanggung jawab atas parsing bahasa lokal.  
**44. Phased Implementation Roadmap**  
- **Fase 1 — **Phase 1: Structural Graph (Symbol, File, Module, Call, Reference, Dependency).  
- **Fase 2 — **Phase 2: Behavioral Graph (CFG, Data Flow, Exception Propagation, Type Resolution).  
- **Fase 3 — **Phase 3: Semantic Graph (Logic Graph, Architecture Rules, Behavioral Diff).  
- **Fase 4 — **Phase 4: Intelligence & AI Layer (Impact Engine, MCP Tools, Context Compression).  
**45. Success Metrics**  
- Akurasi resolusi simbol dan pemanggilan (call resolution) >95%.  
- 30% – 70% penghematan token konteks pada kueri AI coding assistant.  
- Latensi kueri graf di bawah 5 detik (< 5s) pada repository berskala menengah.  
**46. Concrete End-to-End Example**  
| |  
|-|  
| **■ Source Code Contoh **// Source Code Implementasiasync function checkout(orderId: string) {    const order = await getOrder(orderId);    if (!order) throw new OrderNotFound();    const discount = calculateDiscount(order);    const amount = order.total - discount;    try {        const payment = await charge(amount);        if (payment.success) await saveOrder(order, payment);        return payment;    } catch (error) {        await rollback(order);        throw error;    }} |   
   
| |  
|-|  
| **■ Struktur Canonical Code Logic Graph yang Dihasilkan **checkout()├── calls ──> getOrder() ──flows_to──> order├── condition ──> order == null ──true──> throw OrderNotFound├── calls ──> calculateDiscount(order) ──reads──> order├── derives ──> amount ──reads──> order.total & discount├── calls ──> charge(amount) ──flows_to──> payment├── condition ──> payment.success ──true──> saveOrder(order, payment)├── catches ──> error ──calls──> rollback(order) & re-throw└── returns ──> payment |   
   
**47. Hubungan Arsitektural dengan Engineering Memory**  
CCLG adalah Code Intelligence Engine teknis; Engineering Memory adalah Cognitive Knowledge Layer di atasnya:  
| |  
|-|  
| **■ Sinergi Fundamental Dua Pilar **CCLG menjawab:           "BAGAIMANA software bekerja?" (How it executes & flows)Engineering Memory menjawab: "MENGAPA software bekerja seperti itu?" (Why it was built that way) |   
   
**48. Product Definition Final (CCLG)**  
Canonical Code Logic Graph adalah semantic intermediate representation yang memetakan struktur, symbol, execution, control flow, data flow, dependency, logic, external interaction, provenance, dan evolusi source code ke dalam satu graph canonical yang language-agnostic.  
Tujuan akhirnya bukan menghasilkan visualisasi grafis semata, melainkan membuat perangkat lunak dapat dipahami secara presisi oleh manusia maupun mesin melalui pipeline:  
| |  
|-|  
| **■ Pipeline End-to-End Codebase Intelligence **Source Code  ──>  Parser  ──>  Canonical Graph  ──>  Semantic Analysis  ──>  Behavior Model  ──>  Impact Analysis  ──>  AI Context  ──>  Engineering Memory |   
   

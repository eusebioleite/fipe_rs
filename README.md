# 🚗🦀 FIPE_rs
A high-performance vehicle data crawler built with Rust. It extracts information from the Brazilian FIPE Table and persists it into a local SQLite database.

## 🚀 Getting Started

### Prerequisites
- **Rust**
  
### Installation
```bash
git clone [<your-repo-url>](https://github.com/eusebioleite/fipe_rs)
cd fipe_rs-master
```

### Running the App
```bash
cargo run --release
```

## Interface Guide
The application runs as a terminal-based interactive menu:

**Option 1:** Initialize/Reset Database (fipe_rs.db).

**Option 2:** Sync Reference Months.

**Option 3:** Fetch Brands (Car, Motorcycle, Truck).

**Option 4:** Fetch Models for all stored brands.

**Option 5:** Fetch specific Years types for models.

**Option 9:** Full Sync (Runs all steps sequentially).

**Option 0:** Safe Exit.

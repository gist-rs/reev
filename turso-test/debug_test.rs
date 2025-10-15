use std::process::Command;

fn main() {
    let output = Command::new("sqlite3")
        .arg(":memory:")
        .arg("CREATE TABLE test (id TEXT PRIMARY KEY, name TEXT);
               INSERT INTO test VALUES (\"same-id\", \"first\");
               INSERT INTO test VALUES (\"same-id\", \"second\") ON CONFLICT(id) DO UPDATE SET name = excluded.name;
               SELECT COUNT(*) FROM test;
               SELECT id, name FROM test;")
        .output()
        .expect("Failed to run sqlite3");

    let result = String::from_utf8_lossy(&output.stdout);
    println!("Raw result: {:?}", result);
    
    let lines: Vec<&str> = result.lines().collect();
    println!("Lines: {:?}", lines);
    
    let has_count_one = lines.iter().any(|l| l.trim() == "1");
    let has_updated_record = result.contains("same-id|second");
    
    println!("Has count 1: {}", has_count_one);
    println!("Has updated record: {}", has_updated_record);
    
    if has_count_one && has_updated_record {
        println!("✅ SUCCESS: Pure SQLite ON CONFLICT works");
    } else {
        println!("❌ FAILURE: Pure SQLite ON CONFLICT failed");
    }
}

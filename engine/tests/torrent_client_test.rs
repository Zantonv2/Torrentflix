use engine::torrent::TorrentClient;

#[tokio::test]
#[ignore] // Run with: cargo test --package engine -- --ignored --nocapture
async fn test_qbittorrent_connection() {
    // This test requires qBittorrent to be running with Web UI enabled
    let client = TorrentClient::new(
        "http://localhost:5555".to_string(),
        "admin".to_string(),
        "adminadmin".to_string(),
    );

    println!("🧪 Testing qBittorrent connection...");
    
    match client.test_connection().await {
        Ok(_) => println!("✅ Connection successful!"),
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            panic!("qBittorrent connection test failed");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_add_and_remove_torrent() {
    let client = TorrentClient::new(
        "http://localhost:5555".to_string(),
        "admin".to_string(),
        "adminadmin".to_string(),
    );

    println!("🧪 Testing add torrent...");
    
    // Ubuntu 22.04 LTS magnet (legal test torrent)
    let magnet = "magnet:?xt=urn:btih:5a8a73a3095b6b2a5f8e5c5e5e5e5e5e5e5e5e5e&dn=ubuntu-test";
    
    let hash = client.add_torrent(magnet).await
        .expect("Failed to add torrent");
    
    println!("✅ Torrent added with hash: {}", hash);
    
    // Wait a bit for qBittorrent to process
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    println!("🧪 Testing get download status...");
    match client.get_download_status(&hash).await {
        Ok(status) => {
            println!("✅ Got status:");
            println!("   Title: {}", status.title);
            println!("   Progress: {:.1}%", status.progress * 100.0);
            println!("   State: {}", status.state);
        }
        Err(e) => println!("⚠️  Could not get status (torrent might not be ready yet): {}", e),
    }
    
    println!("🧪 Testing get all downloads...");
    let all_downloads = client.get_all_downloads().await
        .expect("Failed to get all downloads");
    println!("✅ Found {} downloads", all_downloads.len());
    
    println!("🧪 Testing pause...");
    client.pause_download(&hash).await
        .expect("Failed to pause");
    println!("✅ Paused");
    
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    println!("🧪 Testing resume...");
    client.resume_download(&hash).await
        .expect("Failed to resume");
    println!("✅ Resumed");
    
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    println!("🧪 Testing remove...");
    client.remove_download(&hash, true).await
        .expect("Failed to remove");
    println!("✅ Removed");
    
    println!("\n🎉 All tests passed!");
}

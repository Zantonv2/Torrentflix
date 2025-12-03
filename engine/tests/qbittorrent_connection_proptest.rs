// Property-based tests for qBittorrent connection validation
// Feature: settings-ui, Property 4: qBittorrent Connection Validation
// Validates: Requirements 2.2, 2.3, 2.4

use anyhow::Result;
use engine::database::Database;
use engine::settings::manager::SettingsManager;
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Enum representing different connection scenarios
#[derive(Debug, Clone)]
enum ConnectionScenario {
    Success,
    InvalidCredentials,
    ConnectionRefused,
    Timeout,
    InvalidUrl,
}

// Generator for valid hostnames
fn valid_hostname() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("localhost".to_string()),
        Just("127.0.0.1".to_string()),
        "[a-z]{3,10}\\.[a-z]{3,10}\\.[a-z]{2,4}".prop_map(|s| s.to_string()),
    ]
}

// Generator for valid port numbers (1024-65535, avoiding privileged ports)
fn valid_port() -> impl Strategy<Value = u16> {
    1024u16..=65535u16
}

// Generator for usernames
fn username() -> impl Strategy<Value = String> {
    "[a-z]{3,15}".prop_map(|s| s.to_string())
}

// Generator for passwords
fn password() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{6,20}".prop_map(|s| s.to_string())
}

// Generator for invalid URLs
fn invalid_url() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("not-a-url".to_string()),
        Just("ftp://invalid-protocol.com".to_string()),
        Just("http://".to_string()),
        Just("://missing-protocol.com".to_string()),
    ]
}

// Generator for connection scenarios (excluding timeout for main test)
fn connection_scenario() -> impl Strategy<Value = ConnectionScenario> {
    prop_oneof![
        Just(ConnectionScenario::Success),
        Just(ConnectionScenario::InvalidCredentials),
        Just(ConnectionScenario::ConnectionRefused),
        Just(ConnectionScenario::InvalidUrl),
    ]
}

// Mock qBittorrent server that responds based on scenario
async fn start_mock_server(
    scenario: ConnectionScenario,
    expected_username: String,
    expected_password: String,
) -> Result<(String, tokio::task::JoinHandle<()>)> {
    // Bind to a random available port
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let url = format!("http://{}", addr);

    let handle = tokio::spawn(async move {
        // Accept one connection
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buffer = vec![0u8; 4096];
            
            match scenario {
                ConnectionScenario::Success => {
                    // Read the request
                    if let Ok(n) = socket.read(&mut buffer).await {
                        let request = String::from_utf8_lossy(&buffer[..n]);
                        
                        // Check if it's a login request
                        if request.contains("POST /api/v2/auth/login") {
                            // Parse form data to check credentials
                            let body_start = request.find("\r\n\r\n").unwrap_or(0) + 4;
                            let body = &request[body_start..];
                            
                            let has_correct_username = body.contains(&format!("username={}", expected_username));
                            let has_correct_password = body.contains(&format!("password={}", expected_password));
                            
                            if has_correct_username && has_correct_password {
                                // Send success response
                                let response = "HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nOk.";
                                let _ = socket.write_all(response.as_bytes()).await;
                            } else {
                                // Send forbidden response
                                let response = "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n";
                                let _ = socket.write_all(response.as_bytes()).await;
                            }
                        }
                    }
                }
                ConnectionScenario::InvalidCredentials => {
                    // Read the request
                    if let Ok(_) = socket.read(&mut buffer).await {
                        // Always send forbidden response
                        let response = "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n";
                        let _ = socket.write_all(response.as_bytes()).await;
                    }
                }
                ConnectionScenario::Timeout => {
                    // Read the request but never respond (simulate timeout)
                    let _ = socket.read(&mut buffer).await;
                    // Sleep for longer than the client timeout (client uses 10s, we wait 12s)
                    tokio::time::sleep(tokio::time::Duration::from_secs(12)).await;
                }
                ConnectionScenario::ConnectionRefused => {
                    // Close the socket immediately without responding
                    drop(socket);
                }
                ConnectionScenario::InvalidUrl => {
                    // This scenario is handled by the client, not the server
                }
            }
        }
    });

    Ok((url, handle))
}

// Helper function to create a test settings manager
async fn create_test_manager() -> Result<(SettingsManager, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_qb_proptest.db");
    let db_url = db_path.to_string_lossy().to_string();

    let db = Database::new(&db_url).await?;
    let settings_db = Arc::new(engine::database::SettingsDatabase::new(Arc::new(db)));
    settings_db.initialize().await?;

    let manager = SettingsManager::new(settings_db);
    Ok((manager, temp_dir))
}

#[cfg(test)]
mod proptest_tests {
    use super::*;

    // Property 4: qBittorrent Connection Validation
    // For any qBittorrent URL, username, and password combination, the test connection
    // operation should either succeed (returning success) or fail with a specific error
    // message (connection refused, invalid credentials, timeout, etc.).
    // Validates: Requirements 2.2, 2.3, 2.4
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]  // Reduced from 100 due to timeout tests
        
        #[test]
        fn test_qbittorrent_connection_validation_property(
            _seed in 0u64..1000000u64
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create test manager
                let (manager, _temp_dir) = create_test_manager().await.unwrap();
                
                // Generate test data using the seed
                let mut runner = proptest::test_runner::TestRunner::new(
                    ProptestConfig::with_cases(1)
                );
                
                // Generate scenario
                let scenario_strategy = connection_scenario();
                let scenario = scenario_strategy.new_tree(&mut runner).unwrap().current();
                
                // Generate credentials
                let username_strategy = username();
                let username_val = username_strategy.new_tree(&mut runner).unwrap().current();
                
                let password_strategy = password();
                let password_val = password_strategy.new_tree(&mut runner).unwrap().current();
                
                match scenario {
                    ConnectionScenario::Success => {
                        // Start mock server that will accept the connection
                        let (url, _handle) = start_mock_server(
                            ConnectionScenario::Success,
                            username_val.clone(),
                            password_val.clone(),
                        ).await.unwrap();
                        
                        // Test connection - should succeed
                        let result = manager.test_qbittorrent_connection(
                            &url,
                            &username_val,
                            &password_val,
                        ).await;
                        
                        assert!(result.is_ok(), 
                            "Connection should succeed for valid credentials, got: {:?}", result);
                    }
                    
                    ConnectionScenario::InvalidCredentials => {
                        // Start mock server that will reject credentials
                        let (url, _handle) = start_mock_server(
                            ConnectionScenario::InvalidCredentials,
                            username_val.clone(),
                            password_val.clone(),
                        ).await.unwrap();
                        
                        // Test connection - should fail with invalid credentials error
                        let result = manager.test_qbittorrent_connection(
                            &url,
                            &username_val,
                            &password_val,
                        ).await;
                        
                        assert!(result.is_err(), 
                            "Connection should fail for invalid credentials");
                        
                        let error_msg = result.unwrap_err().to_string();
                        assert!(
                            error_msg.contains("Invalid credentials") || 
                            error_msg.contains("Forbidden") ||
                            error_msg.contains("403"),
                            "Error should mention invalid credentials, got: {}", error_msg
                        );
                    }
                    
                    ConnectionScenario::ConnectionRefused => {
                        // Use a port that's not listening
                        let url = "http://127.0.0.1:9999";
                        
                        // Test connection - should fail with connection refused error
                        let result = manager.test_qbittorrent_connection(
                            url,
                            &username_val,
                            &password_val,
                        ).await;
                        
                        assert!(result.is_err(), 
                            "Connection should fail when server is not reachable");
                        
                        let error_msg = result.unwrap_err().to_string();
                        assert!(
                            error_msg.contains("Connection refused") || 
                            error_msg.contains("connect") ||
                            error_msg.contains("refused"),
                            "Error should mention connection refused, got: {}", error_msg
                        );
                    }
                    

                    ConnectionScenario::InvalidUrl => {
                        // Generate an invalid URL
                        let invalid_url_strategy = invalid_url();
                        let url = invalid_url_strategy.new_tree(&mut runner).unwrap().current();
                        
                        // Test connection - should fail with URL validation error
                        let result = manager.test_qbittorrent_connection(
                            &url,
                            &username_val,
                            &password_val,
                        ).await;
                        
                        assert!(result.is_err(), 
                            "Connection should fail for invalid URL");
                        
                        let error_msg = result.unwrap_err().to_string();
                        assert!(
                            error_msg.contains("URL") || 
                            error_msg.contains("url") ||
                            error_msg.contains("invalid") ||
                            error_msg.contains("format"),
                            "Error should mention URL validation, got: {}", error_msg
                        );
                    }
                    
                    ConnectionScenario::Timeout => {
                        // This scenario is tested separately due to slowness
                        // Skip in the main property test
                    }
                }
            });
        }
    }

    // Additional test: Verify that successful connections return Ok(())
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]
        
        #[test]
        fn test_successful_connection_returns_ok(
            username_val in username(),
            password_val in password(),
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                let (manager, _temp_dir) = create_test_manager().await.unwrap();
                
                // Start mock server that accepts connections
                let (url, _handle) = start_mock_server(
                    ConnectionScenario::Success,
                    username_val.clone(),
                    password_val.clone(),
                ).await.unwrap();
                
                // Test connection
                let result = manager.test_qbittorrent_connection(
                    &url,
                    &username_val,
                    &password_val,
                ).await;
                
                assert!(result.is_ok(), 
                    "Valid connection should return Ok(()), got: {:?}", result);
            });
        }
    }

    // Additional test: Verify timeout handling (fewer iterations due to slowness)
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]
        
        #[test]
        fn test_timeout_handling(
            username_val in username(),
            password_val in password(),
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                let (manager, _temp_dir) = create_test_manager().await.unwrap();
                
                // Start mock server that will timeout
                let (url, _handle) = start_mock_server(
                    ConnectionScenario::Timeout,
                    username_val.clone(),
                    password_val.clone(),
                ).await.unwrap();
                
                // Test connection - should fail with timeout error
                let result = manager.test_qbittorrent_connection(
                    &url,
                    &username_val,
                    &password_val,
                ).await;
                
                assert!(result.is_err(), 
                    "Connection should fail on timeout");
                
                let error_msg = result.unwrap_err().to_string();
                assert!(
                    error_msg.contains("timeout") || 
                    error_msg.contains("Timeout") ||
                    error_msg.contains("timed out"),
                    "Error should mention timeout, got: {}", error_msg
                );
            });
        }
    }

    // Additional test: Verify that wrong credentials always fail
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]
        
        #[test]
        fn test_wrong_credentials_always_fail(
            correct_username in username(),
            correct_password in password(),
            wrong_username in username(),
            wrong_password in password(),
        ) {
            // Skip if credentials happen to match
            prop_assume!(correct_username != wrong_username || correct_password != wrong_password);
            
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                let (manager, _temp_dir) = create_test_manager().await.unwrap();
                
                // Start mock server with correct credentials
                let (url, _handle) = start_mock_server(
                    ConnectionScenario::Success,
                    correct_username.clone(),
                    correct_password.clone(),
                ).await.unwrap();
                
                // Test connection with wrong credentials
                let result = manager.test_qbittorrent_connection(
                    &url,
                    &wrong_username,
                    &wrong_password,
                ).await;
                
                assert!(result.is_err(), 
                    "Wrong credentials should fail");
                
                let error_msg = result.unwrap_err().to_string();
                assert!(
                    error_msg.contains("Invalid credentials") || 
                    error_msg.contains("Forbidden") ||
                    error_msg.contains("403"),
                    "Error should indicate authentication failure, got: {}", error_msg
                );
            });
        }
    }
}

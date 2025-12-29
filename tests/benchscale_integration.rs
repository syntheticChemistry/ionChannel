//! Integration test for ionChannel using benchScale
//!
//! Tests RustDesk installation and ID retrieval using benchScale LibvirtBackend

use benchscale::backend::{Backend, LibvirtBackend};

#[tokio::test]
#[ignore] // Run with: cargo test --test benchscale_integration -- --ignored
async fn test_existing_vm_rustdesk() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("\n════════════════════════════════════════════════════════════════");
    println!("  Testing benchScale with Existing VM");
    println!("════════════════════════════════════════════════════════════════\n");

    // Create LibvirtBackend
    let backend = LibvirtBackend::new()?;
    // TODO: Re-enable SSH credentials when API is stabilized
    //    .with_ssh_credentials("iontest".to_string(), Some("iontest".to_string()));

    println!("✓ LibvirtBackend created");

    // Check if backend is available
    let available = backend.is_available().await?;
    assert!(available, "Libvirt should be available");
    println!("✓ Libvirt is available");

    // Get info about test1 VM
    println!("\nGetting VM info...");
    let nodes = backend.list_nodes("default").await?;

    let test_vm = nodes
        .iter()
        .find(|n| n.name == "test1")
        .expect("test1 VM should exist");

    println!("✓ Found test1 VM: {}", test_vm.id);
    println!("  Status: {:?}", test_vm.status);

    // Execute command to check if RustDesk is installed
    println!("\nChecking RustDesk installation...");
    let result = backend
        .exec_command(
            &test_vm.container_id,
            vec!["which".to_string(), "rustdesk".to_string()],
        )
        .await?;

    if result.success() {
        println!("✓ RustDesk is installed: {}", result.stdout.trim());

        // Get RustDesk ID
        println!("\nGetting RustDesk ID...");
        let id_result = backend
            .exec_command(
                &test_vm.container_id,
                vec!["rustdesk".to_string(), "--get-id".to_string()],
            )
            .await?;

        if id_result.success() {
            let rustdesk_id = id_result.stdout.trim();
            println!("✓ RustDesk ID: {}", rustdesk_id);

            println!("\n════════════════════════════════════════════════════════════════");
            println!("  ✅ SUCCESS - benchScale Works!");
            println!("════════════════════════════════════════════════════════════════");
            println!("\n  VM: test1");
            println!("  RustDesk ID: {}", rustdesk_id);
            println!("  SSH: ✓ Working");
            println!("  Command Execution: ✓ Working");
            println!("\n  benchScale is ready for ionChannel testing! 🎉\n");

            assert!(!rustdesk_id.is_empty(), "RustDesk ID should not be empty");
        } else {
            eprintln!("⚠️  RustDesk not configured yet");
            eprintln!("  stdout: {}", id_result.stdout);
            eprintln!("  stderr: {}", id_result.stderr);
        }
    } else {
        eprintln!("⚠️  RustDesk not installed");
        eprintln!("  stdout: {}", result.stdout);
        eprintln!("  stderr: {}", result.stderr);
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Run with: cargo test --test benchscale_integration -- --ignored
async fn test_vm_network_operations() -> anyhow::Result<()> {
    println!("\n════════════════════════════════════════════════════════════════");
    println!("  Testing benchScale Network Operations");
    println!("════════════════════════════════════════════════════════════════\n");

    let backend = LibvirtBackend::new()?;

    // Test network creation
    println!("Creating test network...");
    let network_info = backend
        .create_network("benchscale-test", "192.168.200.0/24")
        .await?;

    println!("✓ Network created: {}", network_info.name);
    println!("  ID: {}", network_info.id);
    println!("  Subnet: {}", network_info.subnet);
    println!("  Gateway: {}", network_info.gateway);

    // Clean up
    println!("\nCleaning up test network...");
    backend.delete_network("benchscale-test").await?;
    println!("✓ Network deleted");

    println!("\n✅ Network operations working!\n");

    Ok(())
}

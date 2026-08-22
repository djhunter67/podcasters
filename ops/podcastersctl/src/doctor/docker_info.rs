pub fn is_container_reachable() -> anyhow::Result<bool> {
    let output = std::process::Command::new("docker")
        .arg("info")
        .output()
        .expect("Failed to execute docker command");

    println!(
        "Docker info output: {:#?}",
        String::from_utf8_lossy(&output.stdout)
    );

    Ok(output.status.success())
}

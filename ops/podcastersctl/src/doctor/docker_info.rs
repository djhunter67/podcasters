pub fn can_user_create_containers() -> anyhow::Result<bool> {
    let user_group = std::process::Command::new("groups").output()?;

    Ok(String::from_utf8_lossy(&user_group.stdout).contains("docker"))
}

pub fn is_container_reachable() -> anyhow::Result<bool> {
    let output = std::process::Command::new("docker").arg("info").output()?;

    println!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(output.status.success())
}

pub fn can_user_create_containers() -> anyhow::Result<bool> {
    let user_group = match std::process::Command::new("groups").output() {
        Ok(val) => val,
        Err(err) => {
            return Err(anyhow::Error::msg(format!(
                "Unable to get the user's groups: {err}"
            )));
        }
    };

    Ok(String::from_utf8_lossy(&user_group.stdout).contains("docker"))
}

pub fn is_container_reachable() -> anyhow::Result<bool> {
    let output = match std::process::Command::new("docker").arg("info").output() {
        Ok(val) => val,
        Err(err) => {
            return Err(anyhow::Error::msg(format!("Unable to reach Docker: {err}")));
        }
    };

    // println!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(output.status.success())
}

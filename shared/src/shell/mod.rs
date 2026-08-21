use std::{env, fs, io::Read};

pub fn find_worspace_root() -> anyhow::Result<String> {
    let wkspc_root = env::current_dir()?;

    let mut workspace_root = String::new();

    for root in wkspc_root.ancestors() {
        let dir_files = fs::read_dir(root)?;

        for file in dir_files {
            let entry = file?;

            if entry.file_name().eq("Cargo.toml") {
                let mut file_buf: [u8; 12] = [0; 12]; // Only need the first line

                let mut file = fs::File::open(root.join(entry.file_name()))?;

                let () = file.read_exact(&mut file_buf)?;

                let contents = String::from_utf8_lossy(&file_buf).to_string();
                drop(file);

                contents.split('\n').next().map_or_else(
                    || {
                        eprintln!("Unable to parse the first line");
                        ""
                    },
                    |first_part| first_part,
                );

                if contents.contains("[workspace]") {
                    workspace_root = root.to_string_lossy().to_string();
                    break;
                }
            }
        }
    }

    Ok(workspace_root)
}

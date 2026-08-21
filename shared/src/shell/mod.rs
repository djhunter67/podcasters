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

pub fn get_versioned(app: &str) -> anyhow::Result<String> {
    // std::env!("CARGO_PKG_VERSION").to_string()
    let workspace_root = find_worspace_root()?;
    let mut version: String = String::new();

    let path = std::path::Path::new(&workspace_root).read_dir()?;
    for dir in path {
        let dir = dir?;
        // println!("File: {}", dir.dir_name().to_string_lossy());
        if dir.file_name().eq(app) {
            for file in dir.path().read_dir()? {
                let file = file?;

                if file.file_name().eq("Cargo.toml") {
                    // println!("files: {}", file?.file_name().to_string_lossy());
                    let mut file_buf: Vec<u8> = vec![];

                    // Two directories down is the target file
                    let mut cargo = std::fs::File::open(
                        std::path::Path::new(&workspace_root)
                            .join(dir.file_name())
                            .join(file.file_name()),
                    )?;

                    cargo.read_to_end(&mut file_buf)?;

                    for line in file_buf.split(|lines| *lines == b'\n') {
                        // println!("{:#?}", String::from_utf8_lossy(line));

                        if String::from_utf8_lossy(line).contains("version") {
                            let versioned = line
                                .split(|letter| letter.eq(&b'='))
                                .next_back()
                                .expect("failed to iter the line");

                            version = String::from_utf8_lossy(versioned)
                                .trim()
                                .trim_matches('\"')
                                .to_string();

                            // println!(
                            // "\nVERSION: {:#?}\n",
                            // String::from_utf8_lossy(versioned).trim().trim_matches('\"')
                            // );
                            break; // The target line is at the top of the file
                        }
                    }
                }
            }
        }
    }

    Ok(version)
}

#[cfg(test)]
mod tests {

    use rstest::rstest;

    use super::get_versioned;

    #[rstest]
    fn test_backend_version() {
        assert!(get_versioned("backend").unwrap().starts_with('0'));
    }

    #[rstest]
    fn test_frontend_version() {
        assert!((get_versioned("frontend")).unwrap().starts_with('0'));
    }
}

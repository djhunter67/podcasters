# Bump the version of every crate in a `cargo workpsace`

Manually find and parse every package in a cargo workspace.  Get the current version listed in the `Cargo.toml` file for each crate.  Increment the minor version of every crate `+1` of the current version if the commit on the `trunk` or `main` branch begins w/ `feat:`.  Standard git conventions will be observed.  The `fix:` conventional git keyword will bump the `patch` version `+1` from the current `patch` version.  The minor version will be bumped `+1` if the the git commit on `trunk` or `main` begins with `feat!:`.

This binary will be designed to be called by the `pre-commit` git hook. 

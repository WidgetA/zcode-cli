use codex_utils_absolute_path::AbsolutePathBuf;
use dirs::home_dir;
use std::path::PathBuf;

/// Returns the path to the zcode configuration directory.
///
/// Resolution priority:
///
/// 1. `ZCODE_HOME` environment variable.
/// 2. `CODEX_HOME` environment variable (back-compat with upstream Codex).
/// 3. `~/.zcode` if it exists, or if `~/.codex` does not exist (fresh setups
///    default to `~/.zcode`); otherwise an existing `~/.codex` is respected.
///
/// - If an env var is set, the value must exist and be a directory. The
///   value will be canonicalized and this function will Err otherwise.
/// - If neither env var is set, this function does not verify that the
///   directory exists.
pub fn find_codex_home() -> std::io::Result<AbsolutePathBuf> {
    let zcode_home_env = std::env::var("ZCODE_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    let codex_home_env = std::env::var("CODEX_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    find_codex_home_from_env(zcode_home_env.as_deref(), codex_home_env.as_deref())
}

fn find_codex_home_from_env(
    zcode_home_env: Option<&str>,
    codex_home_env: Option<&str>,
) -> std::io::Result<AbsolutePathBuf> {
    // Honor the `ZCODE_HOME` environment variable first, then `CODEX_HOME`
    // for back-compat, to allow users (and tests) to override the default
    // location.
    let (env_var, val) = match (zcode_home_env, codex_home_env) {
        (Some(val), _) => ("ZCODE_HOME", val),
        (None, Some(val)) => ("CODEX_HOME", val),
        (None, None) => return default_config_home(),
    };
    let path = PathBuf::from(val);
    let metadata = std::fs::metadata(&path).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{env_var} points to {val:?}, but that path does not exist"),
        ),
        _ => std::io::Error::new(
            err.kind(),
            format!("failed to read {env_var} {val:?}: {err}"),
        ),
    })?;

    if !metadata.is_dir() {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{env_var} points to {val:?}, but that path is not a directory"),
        ))
    } else {
        let canonical = path.canonicalize().map_err(|err| {
            std::io::Error::new(
                err.kind(),
                format!("failed to canonicalize {env_var} {val:?}: {err}"),
            )
        })?;
        AbsolutePathBuf::from_absolute_path(canonical)
    }
}

fn default_config_home() -> std::io::Result<AbsolutePathBuf> {
    let home = home_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find home directory",
        )
    })?;
    AbsolutePathBuf::from_absolute_path(default_config_home_dir(&home))
}

/// Picks the default config dir under `home`: prefer `~/.zcode` unless only
/// `~/.codex` exists (an existing upstream Codex install keeps working).
fn default_config_home_dir(home: &std::path::Path) -> PathBuf {
    let zcode_dir = home.join(".zcode");
    let codex_dir = home.join(".codex");
    if zcode_dir.is_dir() || !codex_dir.is_dir() {
        zcode_dir
    } else {
        codex_dir
    }
}

#[cfg(test)]
mod tests {
    use super::default_config_home_dir;
    use super::find_codex_home_from_env;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn find_codex_home_env_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-codex-home");
        let missing_str = missing
            .to_str()
            .expect("missing codex home path should be valid utf-8");

        let err =
            find_codex_home_from_env(None, Some(missing_str)).expect_err("missing CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("CODEX_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_file_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let file_path = temp_home.path().join("codex-home.txt");
        fs::write(&file_path, "not a directory").expect("write temp file");
        let file_str = file_path
            .to_str()
            .expect("file codex home path should be valid utf-8");

        let err = find_codex_home_from_env(None, Some(file_str)).expect_err("file CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp codex home path should be valid utf-8");

        let resolved = find_codex_home_from_env(None, Some(temp_str)).expect("valid CODEX_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn zcode_home_takes_priority_over_codex_home() {
        let zcode_home = TempDir::new().expect("temp zcode home");
        let codex_home = TempDir::new().expect("temp codex home");
        let zcode_str = zcode_home.path().to_str().expect("valid utf-8");
        let codex_str = codex_home.path().to_str().expect("valid utf-8");

        let resolved =
            find_codex_home_from_env(Some(zcode_str), Some(codex_str)).expect("valid ZCODE_HOME");
        let expected = zcode_home
            .path()
            .canonicalize()
            .expect("canonicalize zcode home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
        assert!(
            find_codex_home_from_env(Some(zcode_str), Some("definitely/missing"))
                .is_ok_and(|resolved| resolved == expected),
            "ZCODE_HOME should win even when CODEX_HOME is invalid"
        );
    }

    #[test]
    fn zcode_home_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-zcode-home");
        let missing_str = missing.to_str().expect("valid utf-8");

        let err =
            find_codex_home_from_env(Some(missing_str), None).expect_err("missing ZCODE_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("ZCODE_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn default_config_home_prefers_fresh_zcode_dir() {
        let temp_home = TempDir::new().expect("temp home");
        assert_eq!(
            default_config_home_dir(temp_home.path()),
            temp_home.path().join(".zcode"),
            "fresh setups (neither dir exists) should default to ~/.zcode"
        );

        fs::create_dir(temp_home.path().join(".codex")).expect("create .codex");
        assert_eq!(
            default_config_home_dir(temp_home.path()),
            temp_home.path().join(".codex"),
            "an existing ~/.codex should be respected"
        );

        fs::create_dir(temp_home.path().join(".zcode")).expect("create .zcode");
        assert_eq!(
            default_config_home_dir(temp_home.path()),
            temp_home.path().join(".zcode"),
            "~/.zcode wins when both exist"
        );
    }
}

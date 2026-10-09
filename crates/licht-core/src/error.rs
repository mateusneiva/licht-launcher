use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("HTTP request failed")]
    Http(#[from] reqwest::Error),
    #[error("I/O operation failed")]
    Io(#[from] io::Error),
    #[error("version manifest JSON is invalid")]
    Manifest(#[from] serde_json::Error),
    #[error("version JSON is invalid")]
    Version(#[source] serde_json::Error),
    #[error("version JSON must contain either minecraftArguments or arguments")]
    VersionArguments,
    #[error("asset index JSON is invalid")]
    AssetIndex(#[source] serde_json::Error),
    #[error("downloaded SHA1 does not match")]
    Sha1Mismatch { expected: String, actual: String },
    #[error("download needs at least one attempt")]
    DownloadAttempts,
    #[error("download concurrency must be at least 1")]
    DownloadConcurrency,
    #[error("could not resolve the Licht data directory")]
    DataDirectory,
    #[error("cache path is invalid")]
    CachePath,
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    #[test]
    fn propagating_an_io_error_preserves_its_cause_and_category() {
        fn operation() -> crate::Result<()> {
            Err(io::Error::from(io::ErrorKind::PermissionDenied))?;
            Ok(())
        }

        let error = operation().expect_err("the operation should fail");
        let cause = error
            .source()
            .expect("the original cause should remain available")
            .downcast_ref::<io::Error>()
            .expect("the cause should remain an I/O error");

        assert_eq!(cause.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(error.to_string(), "I/O operation failed");
    }
}

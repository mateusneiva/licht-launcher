use crate::Version;

/// The runtime a version asks for, when the JSON includes `javaVersion`.
pub fn required_runtime(version: &Version) -> Option<&crate::JavaVersion> {
    version.java_version.as_ref()
}

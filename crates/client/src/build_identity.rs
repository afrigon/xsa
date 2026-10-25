use std::fmt;

use crate::BuildProfile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildIdentity {
    Release {
        version: String,
    },
    Local {
        profile: BuildProfile,
        branch: Option<String>,
        commit: Option<String>,
    },
}

impl fmt::Display for BuildIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildIdentity::Release { version } => write!(formatter, "{version}"),
            BuildIdentity::Local {
                profile,
                branch,
                commit,
            } => {
                write!(formatter, "{profile}")?;

                match (branch, commit) {
                    (Some(branch), Some(commit)) => write!(formatter, " · {branch} ({commit})"),
                    (Some(branch), None) => write!(formatter, " · {branch}"),
                    (None, Some(commit)) => write!(formatter, " · {commit}"),
                    (None, None) => Ok(()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local(branch: Option<&str>, commit: Option<&str>) -> BuildIdentity {
        BuildIdentity::Local {
            profile: BuildProfile::Debug,
            branch: branch.map(str::to_string),
            commit: commit.map(str::to_string),
        }
    }

    #[test]
    fn release_shows_only_its_version() {
        let identity = BuildIdentity::Release {
            version: "2026.10.1".to_string(),
        };

        assert_eq!(identity.to_string(), "2026.10.1");
    }

    #[test]
    fn local_shows_profile_branch_and_commit() {
        assert_eq!(
            local(Some("main"), Some("ae20806")).to_string(),
            "debug · main (ae20806)"
        );
        assert_eq!(local(None, Some("ae20806")).to_string(), "debug · ae20806");
        assert_eq!(local(None, None).to_string(), "debug");
    }
}

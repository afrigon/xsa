use xsa_client::{BuildIdentity, BuildProfile};

pub fn current_build() -> BuildIdentity {
    match env!("XSA_BUILD_KIND") {
        "release" => BuildIdentity::Release {
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        "local" => BuildIdentity::Local {
            profile: if cfg!(debug_assertions) {
                BuildProfile::Debug
            } else {
                BuildProfile::Release
            },
            branch: option_env!("XSA_GIT_BRANCH").map(str::to_string),
            commit: option_env!("XSA_GIT_COMMIT").map(str::to_string),
        },
        kind => unreachable!("build.rs emits no build kind {kind}"),
    }
}

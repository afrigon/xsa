use semver::VersionReq;

pub struct Dependency {
    pub pack: String,
    pub requirement: VersionReq,
}

use semver::Version;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Crate {
    pub(crate) name: String,
    pub(crate) version: Version,
    pub(crate) dependency: Vec<CrateDependency>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CrateDependency {
    pub(crate) name: String,
    pub(crate) version: Version,
}

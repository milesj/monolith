use indexmap::IndexMap;
use moon_common::cacheable;
use rustc_hash::FxHashMap;
use schematic::{Config, PathSegment, ValidateError};

cacheable!(
    /// A mapping of file paths and file globs to owners.
    #[derive(Clone, Config, Debug, PartialEq)]
    #[serde(
        untagged,
        expecting = "expected a list of paths, or a map of paths to owners"
    )]
    pub enum OwnersPaths {
        /// A list of file paths. The owner is the `defaultOwner`.
        #[setting(default)]
        List(Vec<String>),
        /// A mapping of file paths to owners.
        Map(IndexMap<String, Vec<String>>),
    }
);

impl OwnersPaths {
    pub fn is_empty(&self) -> bool {
        match self {
            OwnersPaths::List(list) => list.is_empty(),
            OwnersPaths::Map(map) => map.is_empty(),
        }
    }
}

fn validate_paths<C>(
    value: &PartialOwnersPaths,
    data: &PartialOwnersConfig,
    _context: &C,
    _finalize: bool,
) -> Result<(), ValidateError> {
    match value {
        PartialOwnersPaths::List(list) => {
            if !list.is_empty() && data.default_owner.is_none() {
                return Err(ValidateError::new(
                    "a default owner is required when defining a list of paths",
                ));
            }
        }
        PartialOwnersPaths::Map(map) => {
            for (key, value) in map {
                if value.is_empty() && data.default_owner.is_none() {
                    return Err(ValidateError::with_segment(
                        "a default owner is required when defining an empty list of owners",
                        PathSegment::Key(key.to_owned()),
                    ));
                }
            }
        }
    };

    Ok(())
}

fn validate_required_approvals<C>(
    value: &u8,
    _data: &PartialOwnersConfig,
    _context: &C,
    _finalize: bool,
) -> Result<(), ValidateError> {
    if *value == 0 {
        return Err(ValidateError::new("at least 1 approver is required"));
    }

    Ok(())
}

cacheable!(
    /// Defines ownership of source code within the current project, by mapping
    /// file paths and globs to owners. An owner is either a user, team, or group.
    #[derive(Clone, Config, Debug, PartialEq)]
    pub struct OwnersConfig {
        /// Bitbucket only. A mapping of custom groups (prefixed with `@@@`),
        /// to a list of user and normal groups.
        pub custom_groups: FxHashMap<String, Vec<String>>,

        /// The default owner for `paths`.
        pub default_owner: Option<String>,

        /// GitLab only. Marks the code owners section as optional.
        pub optional: bool,

        /// A mapping of file paths and file globs to owners.
        /// When a list, the `defaultOwner` is the owner, and each item is a path.
        /// When an object, the key is a path, and the value is a list of owners.
        #[setting(nested, validate = validate_paths)]
        pub paths: OwnersPaths,

        /// Bitbucket and GitLab only. The number of approvals required for the
        /// request to be satisfied. For Bitbucket, utilizes the `Check()` condition.
        /// For GitLab, marks the code owners section as required.
        #[setting(default = 1, validate = validate_required_approvals)]
        pub required_approvals: u8,
    }
);

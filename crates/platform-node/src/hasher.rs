use moon_hasher::{hash_btree, Digest, Hasher, Sha256};
use moon_lang::LockfileDependencyVersions;
use moon_lang_node::{package::PackageJson, tsconfig::TsConfigJson};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeTargetHasher {
    // Node.js version
    node_version: String,

    // all the dependencies of the project (including dev and peer) and the hashes
    // corresponding with their versions
    dependencies: BTreeMap<String, Vec<String>>,

    // `tsconfig.json` `compilerOptions`
    tsconfig_compiler_options: BTreeMap<String, String>,

    // Version of our hasher
    #[allow(dead_code)]
    version: String,
}

impl NodeTargetHasher {
    pub fn new(node_version: String) -> Self {
        NodeTargetHasher {
            node_version,
            version: String::from("1"),
            ..NodeTargetHasher::default()
        }
    }

    /// Hash `package.json` dependencies as version changes should bust the cache.
    pub fn hash_package_json(
        &mut self,
        package: &PackageJson,
        resolved_deps: &LockfileDependencyVersions,
    ) {
        let copy_deps = |deps: &BTreeMap<String, String>,
                         hashed: &mut BTreeMap<String, Vec<String>>| {
            for (name, version_range) in deps {
                if let Some(resolved_versions) = resolved_deps.get(name) {
                    hashed.insert(name.to_owned(), resolved_versions.to_owned());
                } else {
                    // No match, just use the range itself
                    hashed.insert(name.to_owned(), vec![version_range.to_owned()]);
                }
            }
        };

        if let Some(deps) = &package.dependencies {
            copy_deps(deps, &mut self.dependencies);
        }
        if let Some(dev_deps) = &package.dev_dependencies {
            copy_deps(dev_deps, &mut self.dependencies);
        }
        if let Some(peer_deps) = &package.peer_dependencies {
            copy_deps(peer_deps, &mut self.dependencies);
        }
    }

    /// Hash `tsconfig.json` compiler options that may alter compiled/generated output.
    pub fn hash_tsconfig_json(&mut self, tsconfig: &TsConfigJson) {
        if let Some(compiler_options) = &tsconfig.compiler_options {
            if let Some(module) = &compiler_options.module {
                self.tsconfig_compiler_options
                    .insert("module".to_owned(), format!("{:?}", module));
            }

            if let Some(module_resolution) = &compiler_options.module_resolution {
                self.tsconfig_compiler_options.insert(
                    "module_resolution".to_owned(),
                    format!("{:?}", module_resolution),
                );
            }

            if let Some(target) = &compiler_options.target {
                self.tsconfig_compiler_options
                    .insert("target".to_owned(), format!("{:?}", target));
            }
        }
    }
}

impl Hasher for NodeTargetHasher {
    fn hash(&self, sha: &mut Sha256) {
        sha.update(self.version.as_bytes());
        sha.update(self.node_version.as_bytes());
        for versions in self.dependencies.values() {
            for version in versions {
                sha.update(version.as_bytes());
            }
        }
        hash_btree(&self.tsconfig_compiler_options, sha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moon_hasher::to_hash_only;
    use std::collections::HashMap;

    #[test]
    fn returns_default_hash() {
        let hasher = NodeTargetHasher::new(String::from("0.0.0"));

        assert_eq!(
            to_hash_only(&hasher),
            String::from("ae2cf745a63ca5f47a7218ae5b4a8267295305591457a33a79c46754c1dcce0b")
        );
    }

    #[test]
    fn returns_same_hash_if_called_again() {
        let hasher = NodeTargetHasher::new(String::from("0.0.0"));

        assert_eq!(to_hash_only(&hasher), to_hash_only(&hasher));
    }

    #[test]
    fn returns_different_hash_for_diff_contents() {
        let hasher1 = NodeTargetHasher::new(String::from("0.0.0"));
        let hasher2 = NodeTargetHasher::new(String::from("1.0.0"));

        assert_ne!(to_hash_only(&hasher1), to_hash_only(&hasher2));
    }

    mod btreemap {
        use super::*;

        #[test]
        fn returns_same_hash_for_same_value_inserted() {
            let resolved_deps = HashMap::new();

            let mut package1 = PackageJson::default();
            package1.add_dependency("react", "17.0.0", true);

            let mut hasher1 = NodeTargetHasher::new(String::from("0.0.0"));
            hasher1.hash_package_json(&package1, &resolved_deps);

            let mut hasher2 = NodeTargetHasher::new(String::from("0.0.0"));
            hasher2.hash_package_json(&package1, &resolved_deps);
            hasher2.hash_package_json(&package1, &resolved_deps);

            assert_eq!(to_hash_only(&hasher1), to_hash_only(&hasher2));
        }

        #[test]
        fn returns_same_hash_for_diff_order_insertion() {
            let resolved_deps = HashMap::new();

            let mut package1 = PackageJson::default();
            package1.add_dependency("react", "17.0.0", true);

            let mut package2 = PackageJson::default();
            package2.add_dependency("react-dom", "17.0.0", true);

            let mut hasher1 = NodeTargetHasher::new(String::from("0.0.0"));
            hasher1.hash_package_json(&package2, &resolved_deps);
            hasher1.hash_package_json(&package1, &resolved_deps);

            let mut hasher2 = NodeTargetHasher::new(String::from("0.0.0"));
            hasher2.hash_package_json(&package1, &resolved_deps);
            hasher2.hash_package_json(&package2, &resolved_deps);

            assert_eq!(to_hash_only(&hasher1), to_hash_only(&hasher2));
        }

        #[test]
        fn returns_diff_hash_for_overwritten_value() {
            let resolved_deps = HashMap::new();

            let mut package1 = PackageJson::default();
            package1.add_dependency("react", "17.0.0", true);

            let mut package2 = PackageJson::default();
            package2.add_dependency("react", "18.0.0", true);

            let mut hasher1 = NodeTargetHasher::new(String::from("0.0.0"));
            hasher1.hash_package_json(&package1, &resolved_deps);

            let hash1 = to_hash_only(&hasher1);

            hasher1.hash_package_json(&package2, &resolved_deps);

            let hash2 = to_hash_only(&hasher1);

            assert_ne!(hash1, hash2);
        }
    }

    mod package_json {
        use super::*;

        #[test]
        fn supports_all_dep_types() {
            let resolved_deps = HashMap::new();

            let mut package = PackageJson::default();
            package.add_dependency("moment", "10.0.0", true);

            let mut hasher1 = NodeTargetHasher::new(String::from("0.0.0"));
            hasher1.hash_package_json(&package, &resolved_deps);
            let hash1 = to_hash_only(&hasher1);

            package.dev_dependencies =
                Some(BTreeMap::from([("eslint".to_owned(), "8.0.0".to_owned())]));

            let mut hasher2 = NodeTargetHasher::new(String::from("0.0.0"));
            hasher2.hash_package_json(&package, &resolved_deps);
            let hash2 = to_hash_only(&hasher2);

            package.peer_dependencies =
                Some(BTreeMap::from([("react".to_owned(), "18.0.0".to_owned())]));

            let mut hasher3 = NodeTargetHasher::new(String::from("0.0.0"));
            hasher3.hash_package_json(&package, &resolved_deps);
            let hash3 = to_hash_only(&hasher3);

            assert_ne!(hash1, hash2);
            assert_ne!(hash1, hash3);
            assert_ne!(hash2, hash3);
        }

        #[test]
        fn uses_version_from_resolved_deps() {
            let resolved_deps = HashMap::from([("prettier".to_owned(), vec!["2.1.3".to_owned()])]);

            let mut package = PackageJson::default();
            package.add_dependency("prettier", "^2.0.0", true);
            package.add_dependency("rollup", "^2.0.0", true);

            let mut hasher = NodeTargetHasher::new(String::from("0.0.0"));
            hasher.hash_package_json(&package, &resolved_deps);

            assert_eq!(
                hasher.dependencies,
                BTreeMap::from([
                    ("prettier".to_owned(), vec!["2.1.3".to_owned()]),
                    ("rollup".to_owned(), vec!["^2.0.0".to_owned()])
                ])
            )
        }
    }

    mod tsconfig_json {
        use super::*;

        #[test]
        fn supports_all_dep_types() {
            use moon_lang_node::tsconfig::{CompilerOptions, Module, ModuleResolution, Target};

            let mut tsconfig = TsConfigJson {
                compiler_options: Some(CompilerOptions::default()),
                ..TsConfigJson::default()
            };

            tsconfig.compiler_options.as_mut().unwrap().module = Some(Module::Es2022);

            let mut hasher1 = NodeTargetHasher::new(String::from("0.0.0"));
            hasher1.hash_tsconfig_json(&tsconfig);
            let hash1 = to_hash_only(&hasher1);

            tsconfig
                .compiler_options
                .as_mut()
                .unwrap()
                .module_resolution = Some(ModuleResolution::NodeNext);

            let mut hasher2 = NodeTargetHasher::new(String::from("0.0.0"));
            hasher2.hash_tsconfig_json(&tsconfig);
            let hash2 = to_hash_only(&hasher2);

            tsconfig.compiler_options.as_mut().unwrap().target = Some(Target::Es2019);

            let mut hasher3 = NodeTargetHasher::new(String::from("0.0.0"));
            hasher3.hash_tsconfig_json(&tsconfig);
            let hash3 = to_hash_only(&hasher3);

            assert_ne!(hash1, hash2);
            assert_ne!(hash1, hash3);
            assert_ne!(hash2, hash3);
        }
    }
}

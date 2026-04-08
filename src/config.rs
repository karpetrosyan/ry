use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub packages: HashMap<String, Package>,
    #[serde(default)]
    pub targets: Vec<Target>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Defaults {
    #[serde(default)]
    pub include: Vec<PackageRef>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageRef {
    pub package: String,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub kind: RuleKind,
    #[serde(rename = "match")]
    pub match_pattern: String,
    pub replace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleKind {
    Regex,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TargetVariant {
    File { input: String, output: String },
    Inline { tag: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    #[serde(flatten)]
    pub variant: TargetVariant,
    #[serde(default = "default_inherit_defaults")]
    pub inherit_defaults: bool,
    #[serde(default)]
    pub include: Vec<PackageRef>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

fn default_version() -> u32 {
    1
}

fn default_inherit_defaults() -> bool {
    true
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Config = serde_yaml::from_str(content)?;
        Ok(config)
    }

    pub fn get_effective_rules_from_defaults(&self) -> Vec<Rule> {
        let mut rules = Vec::new();
        let mut seen_ids = HashMap::new();

        for package_ref in &self.defaults.include {
            self.add_package_rules(&mut rules, &mut seen_ids, package_ref);
        }

        rules
    }

    pub fn get_effective_rules(&self, target: &Target) -> Vec<Rule> {
        let mut rules = Vec::new();
        let mut seen_ids = HashMap::new();

        if target.inherit_defaults {
            for package_ref in &self.defaults.include {
                self.add_package_rules(&mut rules, &mut seen_ids, package_ref);
            }
        }

        for package_ref in &target.include {
            self.add_package_rules(&mut rules, &mut seen_ids, package_ref);
        }

        for rule in &target.rules {
            if !seen_ids.contains_key(&rule.id) {
                seen_ids.insert(rule.id.clone(), rules.len());
                rules.push(rule.clone());
            }
        }

        rules
    }

    fn add_package_rules(
        &self,
        rules: &mut Vec<Rule>,
        seen_ids: &mut HashMap<String, usize>,
        package_ref: &PackageRef,
    ) {
        let package = crate::packages::get_package(&package_ref.package, &self.packages);

        if let Some(pkg) = package {
            for rule in &pkg.rules {
                if !package_ref.exclude.contains(&rule.id) && !seen_ids.contains_key(&rule.id) {
                    seen_ids.insert(rule.id.clone(), rules.len());
                    rules.push(rule.clone());
                }
            }
        }
    }
}

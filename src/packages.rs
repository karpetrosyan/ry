use crate::config::{Package, Rule, RuleKind};
use std::collections::HashMap;

pub fn get_builtin_packages() -> HashMap<String, Package> {
    let mut packages = HashMap::new();

    packages.insert("std".to_string(), create_std_package());

    packages
}

fn create_std_package() -> Package {
    Package {
        rules: vec![
            Rule {
                id: Some("async-def".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\basync\s+def\b".to_string(),
                replace: "def".to_string(),
            },
            Rule {
                id: Some("async-with".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\basync\s+with\b".to_string(),
                replace: "with".to_string(),
            },
            Rule {
                id: Some("async-for".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\basync\s+for\b".to_string(),
                replace: "for".to_string(),
            },
            Rule {
                id: Some("strip-await".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\bawait\s+".to_string(),
                replace: "".to_string(),
            },
            Rule {
                id: Some("dunder-anext".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\b__anext__\b".to_string(),
                replace: "__next__".to_string(),
            },
            Rule {
                id: Some("dunder-aenter".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\b__aenter__\b".to_string(),
                replace: "__enter__".to_string(),
            },
            Rule {
                id: Some("dunder-aexit".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\b__aexit__\b".to_string(),
                replace: "__exit__".to_string(),
            },
            Rule {
                id: Some("dunder-aiter".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\b__aiter__\b".to_string(),
                replace: "__iter__".to_string(),
            },
            Rule {
                id: Some("aiter".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\baiter\b".to_string(),
                replace: "iter".to_string(),
            },
            Rule {
                id: Some("async-iterator".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\bAsyncIterator\b".to_string(),
                replace: "Iterator".to_string(),
            },
            Rule {
                id: Some("asynccontextmanager".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\basynccontextmanager\b".to_string(),
                replace: "contextmanager".to_string(),
            },
            Rule {
                id: Some("abstract-async-context-manager".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\bAbstractAsyncContextManager\b".to_string(),
                replace: "AbstractContextManager".to_string(),
            },
            Rule {
                id: Some("from-asyncio-sleep".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"from asyncio import sleep".to_string(),
                replace: "from time import sleep".to_string(),
            },
            Rule {
                id: Some("from-asyncio-condition".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"from asyncio import Condition".to_string(),
                replace: "from threading import Condition".to_string(),
            },
            Rule {
                id: Some("from-asyncio-semaphore".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"from asyncio import Semaphore".to_string(),
                replace: "from threading import Semaphore".to_string(),
            },
            Rule {
                id: Some("from-asyncio-lock".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"from asyncio import Lock".to_string(),
                replace: "from threading import Lock".to_string(),
            },
            Rule {
                id: Some("import-asyncio-to-threading".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"import asyncio".to_string(),
                replace: "import threading".to_string(),
            },
            Rule {
                id: Some("asyncio-condition".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"asyncio\.Condition".to_string(),
                replace: "threading.Condition".to_string(),
            },
            Rule {
                id: Some("asyncio-lock".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"asyncio\.Lock".to_string(),
                replace: "threading.Lock".to_string(),
            },
            Rule {
                id: Some("asyncio-semaphore".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"asyncio\.Semaphore".to_string(),
                replace: "threading.Semaphore".to_string(),
            },
            Rule {
                id: Some("asyncio-timeout-error".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"asyncio\.TimeoutError".to_string(),
                replace: "TimeoutError".to_string(),
            },
            Rule {
                id: Some("pytest-mark-asyncio".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"@pytest\.mark\.asyncio".to_string(),
                replace: "".to_string(),
            },
            Rule {
                id: Some("stop-async-iteration".to_string()),
                kind: RuleKind::Regex,
                match_pattern: r"\bStopAsyncIteration\b".to_string(),
                replace: "StopIteration".to_string(),
            },
        ],
    }
}

pub fn get_package(name: &str, custom_packages: &HashMap<String, Package>) -> Option<Package> {
    if let Some(pkg) = custom_packages.get(name) {
        return Some(Package {
            rules: pkg.rules.clone(),
        });
    }

    let builtins = get_builtin_packages();
    builtins.get(name).map(|pkg| Package {
        rules: pkg.rules.clone(),
    })
}

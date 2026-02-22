use std::fs;
use std::path::Path;

fn make_temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("volki_scanner_{}_{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

mod node {
    use super::*;
    use volki::core::volkiwithstds::collections::String as VString;
    use volki::libs::lang::shared::license::types::{RiskLevel, ScanConfig};

    fn node_config(path: &str) -> ScanConfig {
        ScanConfig {
            path: VString::from(path),
            include_dev: false,
            filter: None,
            exclude: None,
            risk_level: RiskLevel::High,
        }
    }

    #[test]
    fn scan_node_with_node_modules() {
        let dir = make_temp_dir("node_scan");

        // Create package.json
        fs::write(
            dir.join("package.json"),
            r#"{"name": "test-app", "version": "1.0.0", "dependencies": {"lodash": "^4.17.21"}}"#,
        )
        .unwrap();

        // Create node_modules/lodash/package.json
        let lodash_dir = dir.join("node_modules").join("lodash");
        fs::create_dir_all(&lodash_dir).unwrap();
        fs::write(
            lodash_dir.join("package.json"),
            r#"{"name": "lodash", "version": "4.17.21", "license": "MIT"}"#,
        )
        .unwrap();

        let config = node_config(dir.to_str().unwrap());
        let result = volki::libs::lang::js::license::scan(&config).unwrap();

        assert_eq!(result.project_name, "test-app");
        assert_eq!(result.total_packages, 1);
        assert_eq!(result.packages[0].name, "lodash");
        assert_eq!(result.packages[0].license, "MIT");

        cleanup(&dir);
    }

    #[test]
    fn scan_node_no_node_modules() {
        let dir = make_temp_dir("node_no_nm");

        fs::write(
            dir.join("package.json"),
            r#"{"name": "test-app", "version": "1.0.0"}"#,
        )
        .unwrap();

        let config = node_config(dir.to_str().unwrap());
        let result = volki::libs::lang::js::license::scan(&config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{err}").contains("node_modules"));

        cleanup(&dir);
    }
}

mod php {
    use super::*;
    use volki::core::volkiwithstds::collections::String as VString;
    use volki::libs::lang::shared::license::types::{RiskLevel, ScanConfig};

    fn php_config(path: &str) -> ScanConfig {
        ScanConfig {
            path: VString::from(path),
            include_dev: false,
            filter: None,
            exclude: None,
            risk_level: RiskLevel::High,
        }
    }

    #[test]
    fn scan_php_with_lock() {
        let dir = make_temp_dir("php_scan");

        // Create composer.json
        fs::write(
            dir.join("composer.json"),
            r#"{"name": "test/app", "require": {"monolog/monolog": "^3.0"}}"#,
        )
        .unwrap();

        // Create composer.lock with packages array
        fs::write(
            dir.join("composer.lock"),
            r#"{"packages": [{"name": "monolog/monolog", "version": "3.5.0", "license": ["MIT"]}]}"#,
        )
        .unwrap();

        let config = php_config(dir.to_str().unwrap());
        let result = volki::libs::lang::php::license::scan(&config).unwrap();

        assert_eq!(result.total_packages, 1);
        assert_eq!(result.packages[0].name, "monolog/monolog");
        assert_eq!(result.packages[0].license, "MIT");

        cleanup(&dir);
    }
}

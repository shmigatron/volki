use std::io::Write;

use super::types::{LicenseCategory, ScanResult};
use crate::core::cli::style;

fn category_color(cat: LicenseCategory) -> fn(&str) -> String {
    match cat {
        LicenseCategory::Permissive => style::green,
        LicenseCategory::WeakCopyleft => style::yellow,
        LicenseCategory::StrongCopyleft => style::red,
        LicenseCategory::Unknown => style::purple,
    }
}

fn print_header(w: &mut dyn Write, result: &ScanResult) {
    let _ = writeln!(w);
    let _ = writeln!(
        w,
        "  {} {} {}",
        style::bold("license scan:"),
        style::bold(&result.project_name),
        style::dim(&format!("({} packages)", result.total_packages)),
    );
    let _ = writeln!(w);
}

/// Default list view: alphabetical list with colored license labels.
pub fn print_list(w: &mut dyn Write, result: &ScanResult) {
    print_header(w, result);

    if result.packages.is_empty() {
        let _ = writeln!(w, "  {} no packages found", style::dim(style::BULLET));
        let _ = writeln!(w);
        return;
    }

    let max_license_len = result
        .packages
        .iter()
        .map(|p| p.license.len())
        .max()
        .unwrap_or(0);

    for pkg in &result.packages {
        let color_fn = category_color(pkg.category);
        let padded = format!("{:<width$}", pkg.license, width = max_license_len);
        let _ = writeln!(
            w,
            "  {}  {}",
            color_fn(&padded),
            style::dim(&format!("{}@{}", pkg.name, pkg.version)),
        );
    }
    let _ = writeln!(w);
}

/// Grouped view: packages grouped by license type.
pub fn print_grouped(w: &mut dyn Write, result: &ScanResult) {
    print_header(w, result);

    if result.packages.is_empty() {
        let _ = writeln!(w, "  {} no packages found", style::dim(style::BULLET));
        let _ = writeln!(w);
        return;
    }

    // Sort licenses: permissive first, then weak copyleft, strong copyleft, unknown
    let mut license_keys: Vec<&String> = result.by_license.keys().collect();
    license_keys.sort_by(|a, b| {
        let cat_a = LicenseCategory::from_license_str(a);
        let cat_b = LicenseCategory::from_license_str(b);
        let ord = category_sort_key(cat_a).cmp(&category_sort_key(cat_b));
        if ord == std::cmp::Ordering::Equal {
            a.to_lowercase().cmp(&b.to_lowercase())
        } else {
            ord
        }
    });

    for license in license_keys {
        let pkgs = &result.by_license[license];
        let cat = LicenseCategory::from_license_str(license);
        let color_fn = category_color(cat);

        let _ = writeln!(
            w,
            "  {} {}",
            style::bold(&color_fn(license)),
            style::dim(&format!("({} packages)", pkgs.len())),
        );

        let mut sorted_pkgs = pkgs.clone();
        sorted_pkgs.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        for pkg in &sorted_pkgs {
            let _ = writeln!(w, "    {} {}", style::dim(style::TREE_BRANCH), style::dim(pkg));
        }
        let _ = writeln!(w);
    }
}

/// Summary view: count per license with a visual bar chart.
pub fn print_summary(w: &mut dyn Write, result: &ScanResult) {
    print_header(w, result);

    if result.packages.is_empty() {
        let _ = writeln!(w, "  {} no packages found", style::dim(style::BULLET));
        let _ = writeln!(w);
        return;
    }

    let mut entries: Vec<(&String, &Vec<String>)> = result.by_license.iter().collect();
    entries.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let max_count = entries.first().map(|(_, v)| v.len()).unwrap_or(1);
    let max_bar_width = 20;

    let max_name_len = entries.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

    for (license, pkgs) in &entries {
        let count = pkgs.len();
        let cat = LicenseCategory::from_license_str(license);
        let color_fn = category_color(cat);

        let bar_len = if max_count > 0 {
            (count * max_bar_width) / max_count
        } else {
            0
        }
        .max(1);

        let bar: String = "\u{2588}".repeat(bar_len);
        let padded_name = format!("{:<width$}", license, width = max_name_len);

        let _ = writeln!(
            w,
            "  {}  {} {}",
            color_fn(&padded_name),
            color_fn(&bar),
            count,
        );
    }

    let _ = writeln!(w);
    let _ = writeln!(w, "  {}", style::bold("by category:"));

    let categories = [
        LicenseCategory::Permissive,
        LicenseCategory::WeakCopyleft,
        LicenseCategory::StrongCopyleft,
        LicenseCategory::Unknown,
    ];

    for cat in &categories {
        if let Some(pkgs) = result.by_category.get(cat) {
            let color_fn = category_color(*cat);
            let _ = writeln!(
                w,
                "    {} {}: {}",
                style::BULLET,
                color_fn(&cat.to_string()),
                pkgs.len(),
            );
        }
    }

    let _ = writeln!(w);
}

fn category_sort_key(cat: LicenseCategory) -> u8 {
    match cat {
        LicenseCategory::Permissive => 0,
        LicenseCategory::WeakCopyleft => 1,
        LicenseCategory::StrongCopyleft => 2,
        LicenseCategory::Unknown => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::libs::lang::shared::license::types::{LicenseSource, PackageLicense};

    fn make_pkg(name: &str, version: &str, license: &str) -> PackageLicense {
        PackageLicense {
            name: name.to_string(),
            version: version.to_string(),
            license: license.to_string(),
            category: LicenseCategory::from_license_str(license),
            source: LicenseSource::ManifestField,
        }
    }

    fn make_result(project: &str, packages: Vec<PackageLicense>) -> ScanResult {
        let mut by_license: HashMap<String, Vec<String>> = HashMap::new();
        let mut by_category: HashMap<LicenseCategory, Vec<String>> = HashMap::new();
        for pkg in &packages {
            let label = format!("{}@{}", pkg.name, pkg.version);
            by_license.entry(pkg.license.clone()).or_default().push(label.clone());
            by_category.entry(pkg.category).or_default().push(label);
        }
        let total = packages.len();
        ScanResult {
            project_name: project.to_string(),
            total_packages: total,
            packages,
            by_license,
            by_category,
        }
    }

    fn render(f: fn(&mut dyn std::io::Write, &ScanResult), result: &ScanResult) -> String {
        let mut buf = Vec::new();
        f(&mut buf, result);
        String::from_utf8(buf).unwrap()
    }

    // --- print_list ---

    #[test]
    fn list_empty_packages() {
        let result = make_result("test", vec![]);
        let output = render(print_list, &result);
        assert!(output.contains("no packages found"));
    }

    #[test]
    fn list_single_package() {
        let result = make_result("myproject", vec![make_pkg("lodash", "4.17.21", "MIT")]);
        let output = render(print_list, &result);
        assert!(output.contains("lodash"));
        assert!(output.contains("4.17.21"));
        assert!(output.contains("MIT"));
    }

    #[test]
    fn list_header_shows_project_name() {
        let result = make_result("myproject", vec![make_pkg("a", "1.0", "MIT")]);
        let output = render(print_list, &result);
        assert!(output.contains("myproject"));
    }

    // --- print_grouped ---

    #[test]
    fn grouped_empty() {
        let result = make_result("test", vec![]);
        let output = render(print_grouped, &result);
        assert!(output.contains("no packages found"));
    }

    #[test]
    fn grouped_permissive_before_copyleft() {
        let result = make_result("test", vec![
            make_pkg("a", "1.0", "MIT"),
            make_pkg("b", "1.0", "GPL-3.0"),
        ]);
        let output = render(print_grouped, &result);
        let mit_pos = output.find("MIT").unwrap();
        let gpl_pos = output.find("GPL").unwrap();
        assert!(mit_pos < gpl_pos);
    }

    #[test]
    fn grouped_count_per_group() {
        let result = make_result("test", vec![
            make_pkg("a", "1.0", "MIT"),
            make_pkg("b", "2.0", "MIT"),
        ]);
        let output = render(print_grouped, &result);
        assert!(output.contains("2 packages"));
    }

    // --- print_summary ---

    #[test]
    fn summary_empty() {
        let result = make_result("test", vec![]);
        let output = render(print_summary, &result);
        assert!(output.contains("no packages found"));
    }

    #[test]
    fn summary_bar_chart_present() {
        let result = make_result("test", vec![make_pkg("a", "1.0", "MIT")]);
        let output = render(print_summary, &result);
        assert!(output.contains("\u{2588}")); // bar char
    }

    #[test]
    fn summary_category_counts() {
        let result = make_result("test", vec![
            make_pkg("a", "1.0", "MIT"),
            make_pkg("b", "1.0", "GPL-3.0"),
        ]);
        let output = render(print_summary, &result);
        assert!(output.contains("Permissive"));
        assert!(output.contains("Strong Copyleft"));
    }

    #[test]
    fn summary_sorted_by_count_desc() {
        let result = make_result("test", vec![
            make_pkg("a", "1.0", "MIT"),
            make_pkg("b", "1.0", "MIT"),
            make_pkg("c", "1.0", "GPL-3.0"),
        ]);
        let output = render(print_summary, &result);
        // MIT should appear before GPL-3.0 because it has more packages
        let mit_pos = output.find("MIT").unwrap();
        let gpl_pos = output.find("GPL").unwrap();
        assert!(mit_pos < gpl_pos);
    }
}

use std::fs;
use std::path::Path;

use crate::log_debug;

use super::types::*;

fn has_file(dir: &Path, name: &str) -> bool {
    dir.join(name).is_file()
}

fn has_dir(dir: &Path, name: &str) -> bool {
    dir.join(name).is_dir()
}

fn manifest_contains_dep(dir: &Path, manifest: &str, dep: &str) -> bool {
    let path = dir.join(manifest);
    match fs::read_to_string(&path) {
        Ok(content) => content.contains(dep),
        Err(_) => false,
    }
}

pub fn detect(dir: &Path) -> Result<Vec<DetectedProject>, DetectError> {
    if !dir.is_dir() {
        return Err(DetectError::NotADirectory(dir.to_path_buf()));
    }

    log_debug!("scanning {}", dir.display());

    let mut projects = Vec::new();

    let detectors: &[fn(&Path) -> Option<DetectedProject>] = &[
        detect_node,
        detect_python,
        detect_ruby,
        detect_rust,
        detect_go,
        detect_java,
        detect_dotnet,
        detect_php,
        detect_elixir,
        detect_swift,
        detect_dart,
    ];

    for detector in detectors {
        if let Some(project) = detector(dir) {
            log_debug!(
                "detected {} ({}){}",
                project.ecosystem,
                project.manager,
                project.framework.as_ref().map(|f| format!(" [{f}]")).unwrap_or_default()
            );
            projects.push(project);
        }
    }

    log_debug!("found {} project(s)", projects.len());
    Ok(projects)
}

fn detect_node(dir: &Path) -> Option<DetectedProject> {
    if !has_file(dir, "package.json") {
        return None;
    }

    let (manager, lock_file) = if has_file(dir, "bun.lockb") {
        (PackageManager::Bun, Some(dir.join("bun.lockb")))
    } else if has_file(dir, "bun.lock") {
        (PackageManager::Bun, Some(dir.join("bun.lock")))
    } else if has_file(dir, "pnpm-lock.yaml") {
        (PackageManager::Pnpm, Some(dir.join("pnpm-lock.yaml")))
    } else if has_file(dir, "yarn.lock") {
        (PackageManager::Yarn, Some(dir.join("yarn.lock")))
    } else if has_file(dir, "package-lock.json") {
        (PackageManager::Npm, Some(dir.join("package-lock.json")))
    } else {
        (PackageManager::Npm, None)
    };

    let framework = detect_node_framework(dir);

    Some(DetectedProject {
        ecosystem: Ecosystem::Node,
        manager,
        manifest: dir.join("package.json"),
        lock_file,
        framework,
    })
}

fn detect_node_framework(dir: &Path) -> Option<Framework> {
    let m = "package.json";
    if manifest_contains_dep(dir, m, "\"next\"") {
        Some(Framework::NextJs)
    } else if manifest_contains_dep(dir, m, "\"@angular/core\"") {
        Some(Framework::Angular)
    } else if manifest_contains_dep(dir, m, "\"nuxt\"") {
        Some(Framework::Nuxt)
    } else if manifest_contains_dep(dir, m, "\"@sveltejs/kit\"") {
        Some(Framework::SvelteKit)
    } else if manifest_contains_dep(dir, m, "\"svelte\"") {
        Some(Framework::Svelte)
    } else if manifest_contains_dep(dir, m, "\"vue\"") {
        Some(Framework::Vue)
    } else if manifest_contains_dep(dir, m, "\"@nestjs/core\"") {
        Some(Framework::Nest)
    } else if manifest_contains_dep(dir, m, "\"astro\"") {
        Some(Framework::Astro)
    } else if manifest_contains_dep(dir, m, "\"@remix-run/react\"") {
        Some(Framework::Remix)
    } else if manifest_contains_dep(dir, m, "\"gatsby\"") {
        Some(Framework::Gatsby)
    } else if manifest_contains_dep(dir, m, "\"express\"") {
        Some(Framework::Express)
    } else if manifest_contains_dep(dir, m, "\"fastify\"") {
        Some(Framework::Fastify)
    } else if manifest_contains_dep(dir, m, "\"react\"") {
        Some(Framework::React)
    } else {
        None
    }
}

fn detect_python(dir: &Path) -> Option<DetectedProject> {
    let manifest = if has_file(dir, "pyproject.toml") {
        "pyproject.toml"
    } else if has_file(dir, "Pipfile") {
        "Pipfile"
    } else if has_file(dir, "requirements.txt") {
        "requirements.txt"
    } else {
        return None;
    };

    let (manager, lock_file) = if has_file(dir, "uv.lock") {
        (PackageManager::Uv, Some(dir.join("uv.lock")))
    } else if has_file(dir, "poetry.lock") {
        (PackageManager::Poetry, Some(dir.join("poetry.lock")))
    } else if manifest == "Pipfile" {
        let lock = if has_file(dir, "Pipfile.lock") {
            Some(dir.join("Pipfile.lock"))
        } else {
            None
        };
        (PackageManager::Pipenv, lock)
    } else {
        (PackageManager::Pip, None)
    };

    let framework = detect_python_framework(dir, manifest);

    Some(DetectedProject {
        ecosystem: Ecosystem::Python,
        manager,
        manifest: dir.join(manifest),
        lock_file,
        framework,
    })
}

fn detect_python_framework(dir: &Path, manifest: &str) -> Option<Framework> {
    if manifest_contains_dep(dir, manifest, "django") {
        Some(Framework::Django)
    } else if manifest_contains_dep(dir, manifest, "flask") {
        Some(Framework::Flask)
    } else if manifest_contains_dep(dir, manifest, "fastapi") {
        Some(Framework::FastApi)
    } else if manifest_contains_dep(dir, manifest, "tornado") {
        Some(Framework::Tornado)
    } else if manifest_contains_dep(dir, manifest, "pyramid") {
        Some(Framework::Pyramid)
    } else {
        None
    }
}

fn detect_ruby(dir: &Path) -> Option<DetectedProject> {
    if !has_file(dir, "Gemfile") {
        return None;
    }

    let lock_file = if has_file(dir, "Gemfile.lock") {
        Some(dir.join("Gemfile.lock"))
    } else {
        None
    };

    let framework = if has_file(dir, "config/routes.rb") || has_file(dir, "bin/rails") {
        Some(Framework::Rails)
    } else if manifest_contains_dep(dir, "Gemfile", "sinatra") {
        Some(Framework::Sinatra)
    } else if manifest_contains_dep(dir, "Gemfile", "hanami") {
        Some(Framework::Hanami)
    } else {
        None
    };

    Some(DetectedProject {
        ecosystem: Ecosystem::Ruby,
        manager: PackageManager::Bundler,
        manifest: dir.join("Gemfile"),
        lock_file,
        framework,
    })
}

fn detect_rust(dir: &Path) -> Option<DetectedProject> {
    if !has_file(dir, "Cargo.toml") {
        return None;
    }

    let lock_file = if has_file(dir, "Cargo.lock") {
        Some(dir.join("Cargo.lock"))
    } else {
        None
    };

    let framework = detect_rust_framework(dir);

    Some(DetectedProject {
        ecosystem: Ecosystem::Rust,
        manager: PackageManager::Cargo,
        manifest: dir.join("Cargo.toml"),
        lock_file,
        framework,
    })
}

fn detect_rust_framework(dir: &Path) -> Option<Framework> {
    let m = "Cargo.toml";
    if manifest_contains_dep(dir, m, "actix-web") {
        Some(Framework::Actix)
    } else if manifest_contains_dep(dir, m, "axum") {
        Some(Framework::Axum)
    } else if manifest_contains_dep(dir, m, "rocket") {
        Some(Framework::Rocket)
    } else if manifest_contains_dep(dir, m, "tauri") {
        Some(Framework::Tauri)
    } else if manifest_contains_dep(dir, m, "leptos") {
        Some(Framework::Leptos)
    } else if manifest_contains_dep(dir, m, "yew") {
        Some(Framework::Yew)
    } else if manifest_contains_dep(dir, m, "bevy") {
        Some(Framework::Bevy)
    } else {
        None
    }
}

fn detect_go(dir: &Path) -> Option<DetectedProject> {
    if !has_file(dir, "go.mod") {
        return None;
    }

    let lock_file = if has_file(dir, "go.sum") {
        Some(dir.join("go.sum"))
    } else {
        None
    };

    let framework = detect_go_framework(dir);

    Some(DetectedProject {
        ecosystem: Ecosystem::Go,
        manager: PackageManager::GoModules,
        manifest: dir.join("go.mod"),
        lock_file,
        framework,
    })
}

fn detect_go_framework(dir: &Path) -> Option<Framework> {
    let m = "go.mod";
    if manifest_contains_dep(dir, m, "github.com/gin-gonic/gin") {
        Some(Framework::Gin)
    } else if manifest_contains_dep(dir, m, "github.com/labstack/echo") {
        Some(Framework::Echo)
    } else if manifest_contains_dep(dir, m, "github.com/gofiber/fiber") {
        Some(Framework::Fiber)
    } else if manifest_contains_dep(dir, m, "github.com/go-chi/chi") {
        Some(Framework::Chi)
    } else if manifest_contains_dep(dir, m, "github.com/gobuffalo/buffalo") {
        Some(Framework::Buffalo)
    } else {
        None
    }
}

fn detect_java(dir: &Path) -> Option<DetectedProject> {
    if has_file(dir, "build.gradle") || has_file(dir, "build.gradle.kts") {
        let manifest = if has_file(dir, "build.gradle.kts") {
            "build.gradle.kts"
        } else {
            "build.gradle"
        };

        let lock_file = if has_file(dir, "gradle.lockfile") {
            Some(dir.join("gradle.lockfile"))
        } else {
            None
        };

        let framework = detect_java_framework(dir, manifest);

        return Some(DetectedProject {
            ecosystem: Ecosystem::Java,
            manager: PackageManager::Gradle,
            manifest: dir.join(manifest),
            lock_file,
            framework,
        });
    }

    if has_file(dir, "pom.xml") {
        let framework = detect_java_framework(dir, "pom.xml");

        return Some(DetectedProject {
            ecosystem: Ecosystem::Java,
            manager: PackageManager::Maven,
            manifest: dir.join("pom.xml"),
            lock_file: None,
            framework,
        });
    }

    None
}

fn detect_java_framework(dir: &Path, manifest: &str) -> Option<Framework> {
    if manifest_contains_dep(dir, manifest, "spring-boot") {
        Some(Framework::Spring)
    } else if manifest_contains_dep(dir, manifest, "quarkus") {
        Some(Framework::Quarkus)
    } else if manifest_contains_dep(dir, manifest, "micronaut") {
        Some(Framework::Micronaut)
    } else if manifest_contains_dep(dir, manifest, "jakarta.") {
        Some(Framework::Jakarta)
    } else {
        None
    }
}

fn detect_dotnet(dir: &Path) -> Option<DetectedProject> {
    let entries = fs::read_dir(dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "csproj" || ext == "sln" {
                let framework = detect_dotnet_framework(dir, &path);

                return Some(DetectedProject {
                    ecosystem: Ecosystem::DotNet,
                    manager: PackageManager::Nuget,
                    manifest: path,
                    lock_file: None,
                    framework,
                });
            }
        }
    }

    None
}

fn detect_dotnet_framework(dir: &Path, manifest: &Path) -> Option<Framework> {
    let content = fs::read_to_string(manifest).unwrap_or_default();
    if content.contains("Microsoft.AspNetCore") {
        Some(Framework::AspNet)
    } else if content.contains("Blazor") {
        Some(Framework::Blazor)
    } else if content.contains("Microsoft.Maui") {
        Some(Framework::Maui)
    } else {
        let _ = dir;
        None
    }
}

fn detect_php(dir: &Path) -> Option<DetectedProject> {
    if !has_file(dir, "composer.json") {
        return None;
    }

    let lock_file = if has_file(dir, "composer.lock") {
        Some(dir.join("composer.lock"))
    } else {
        None
    };

    let framework = detect_php_framework(dir);

    Some(DetectedProject {
        ecosystem: Ecosystem::Php,
        manager: PackageManager::Composer,
        manifest: dir.join("composer.json"),
        lock_file,
        framework,
    })
}

fn detect_php_framework(dir: &Path) -> Option<Framework> {
    if has_file(dir, "artisan") || manifest_contains_dep(dir, "composer.json", "laravel/framework") {
        Some(Framework::Laravel)
    } else if manifest_contains_dep(dir, "composer.json", "symfony/framework-bundle") {
        Some(Framework::Symfony)
    } else if manifest_contains_dep(dir, "composer.json", "slim/slim") {
        Some(Framework::Slim)
    } else if manifest_contains_dep(dir, "composer.json", "cakephp/cakephp") {
        Some(Framework::CakePhp)
    } else {
        None
    }
}

fn detect_elixir(dir: &Path) -> Option<DetectedProject> {
    if !has_file(dir, "mix.exs") {
        return None;
    }

    let lock_file = if has_file(dir, "mix.lock") {
        Some(dir.join("mix.lock"))
    } else {
        None
    };

    let framework = detect_elixir_framework(dir);

    Some(DetectedProject {
        ecosystem: Ecosystem::Elixir,
        manager: PackageManager::Mix,
        manifest: dir.join("mix.exs"),
        lock_file,
        framework,
    })
}

fn detect_elixir_framework(dir: &Path) -> Option<Framework> {
    let m = "mix.exs";
    if manifest_contains_dep(dir, m, ":phoenix") {
        Some(Framework::Phoenix)
    } else if manifest_contains_dep(dir, m, ":nerves") {
        Some(Framework::Nerves)
    } else {
        None
    }
}

fn detect_swift(dir: &Path) -> Option<DetectedProject> {
    if !has_file(dir, "Package.swift") {
        return None;
    }

    let lock_file = if has_file(dir, "Package.resolved") {
        Some(dir.join("Package.resolved"))
    } else {
        None
    };

    let framework = if manifest_contains_dep(dir, "Package.swift", "vapor") {
        Some(Framework::Vapor)
    } else {
        None
    };

    Some(DetectedProject {
        ecosystem: Ecosystem::Swift,
        manager: PackageManager::Spm,
        manifest: dir.join("Package.swift"),
        lock_file,
        framework,
    })
}

fn detect_dart(dir: &Path) -> Option<DetectedProject> {
    if !has_file(dir, "pubspec.yaml") {
        return None;
    }

    let lock_file = if has_file(dir, "pubspec.lock") {
        Some(dir.join("pubspec.lock"))
    } else {
        None
    };

    let framework =
        if has_file(dir, ".metadata") || has_dir(dir, "android") || has_dir(dir, "ios") {
            Some(Framework::Flutter)
        } else if manifest_contains_dep(dir, "pubspec.yaml", "angular") {
            Some(Framework::AngularDart)
        } else {
            None
        };

    Some(DetectedProject {
        ecosystem: Ecosystem::Dart,
        manager: PackageManager::Pub,
        manifest: dir.join("pubspec.yaml"),
        lock_file,
        framework,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn make_temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("volki_detect_{}_{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn touch(dir: &Path, name: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&path, "").unwrap();
    }

    fn write_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&path, content).unwrap();
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detect_volki_project() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let projects = detect(dir).expect("detection should succeed");

        assert_eq!(projects.len(), 1);
        let project = &projects[0];
        assert_eq!(project.ecosystem, Ecosystem::Rust);
        assert_eq!(project.manager, PackageManager::Cargo);
        assert!(project.manifest.ends_with("Cargo.toml"));
        assert!(project.lock_file.as_ref().unwrap().ends_with("Cargo.lock"));
        assert!(project.framework.is_none());
    }

    #[test]
    fn detect_empty_dir() {
        let dir = make_temp_dir("empty");
        let projects = detect(&dir).unwrap();
        assert!(projects.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn detect_nonexistent_dir() {
        let dir = std::env::temp_dir().join("volki_nonexistent_dir_abc123");
        assert!(detect(&dir).is_err());
    }

    #[test]
    fn detect_file_not_dir() {
        let dir = make_temp_dir("fileasdir");
        let file_path = dir.join("afile");
        fs::write(&file_path, "").unwrap();
        assert!(detect(&file_path).is_err());
        cleanup(&dir);
    }

    // --- Node ---

    #[test]
    fn detect_node_npm() {
        let dir = make_temp_dir("node_npm");
        touch(&dir, "package.json");
        touch(&dir, "package-lock.json");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].ecosystem, Ecosystem::Node);
        assert_eq!(projects[0].manager, PackageManager::Npm);
        assert!(projects[0].lock_file.is_some());
        cleanup(&dir);
    }

    #[test]
    fn detect_node_yarn() {
        let dir = make_temp_dir("node_yarn");
        touch(&dir, "package.json");
        touch(&dir, "yarn.lock");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].manager, PackageManager::Yarn);
        cleanup(&dir);
    }

    #[test]
    fn detect_node_pnpm() {
        let dir = make_temp_dir("node_pnpm");
        touch(&dir, "package.json");
        touch(&dir, "pnpm-lock.yaml");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].manager, PackageManager::Pnpm);
        cleanup(&dir);
    }

    #[test]
    fn detect_node_bun_lockb() {
        let dir = make_temp_dir("node_bunlb");
        touch(&dir, "package.json");
        touch(&dir, "bun.lockb");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].manager, PackageManager::Bun);
        cleanup(&dir);
    }

    #[test]
    fn detect_node_bun_lock() {
        let dir = make_temp_dir("node_bunl");
        touch(&dir, "package.json");
        touch(&dir, "bun.lock");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].manager, PackageManager::Bun);
        cleanup(&dir);
    }

    #[test]
    fn detect_node_no_lock() {
        let dir = make_temp_dir("node_nolock");
        touch(&dir, "package.json");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].manager, PackageManager::Npm);
        assert!(projects[0].lock_file.is_none());
        cleanup(&dir);
    }

    #[test]
    fn detect_node_bun_priority_over_yarn() {
        let dir = make_temp_dir("node_bunprio");
        touch(&dir, "package.json");
        touch(&dir, "bun.lockb");
        touch(&dir, "yarn.lock");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].manager, PackageManager::Bun);
        cleanup(&dir);
    }

    #[test]
    fn detect_node_nextjs() {
        let dir = make_temp_dir("node_next");
        write_file(&dir, "package.json", r#"{"dependencies": {"next": "^14.0.0", "react": "^18"}}"#);
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::NextJs));
        cleanup(&dir);
    }

    #[test]
    fn detect_node_react() {
        let dir = make_temp_dir("node_react");
        write_file(&dir, "package.json", r#"{"dependencies": {"react": "^18.0.0"}}"#);
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::React));
        cleanup(&dir);
    }

    #[test]
    fn detect_node_angular() {
        let dir = make_temp_dir("node_angular");
        write_file(&dir, "package.json", r#"{"dependencies": {"@angular/core": "^17"}}"#);
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Angular));
        cleanup(&dir);
    }

    #[test]
    fn detect_node_express() {
        let dir = make_temp_dir("node_express");
        write_file(&dir, "package.json", r#"{"dependencies": {"express": "^4"}}"#);
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Express));
        cleanup(&dir);
    }

    // --- Python ---

    #[test]
    fn detect_python_pyproject() {
        let dir = make_temp_dir("py_pyproj");
        touch(&dir, "pyproject.toml");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Python);
        cleanup(&dir);
    }

    #[test]
    fn detect_python_pipfile() {
        let dir = make_temp_dir("py_pipfile");
        touch(&dir, "Pipfile");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Python);
        assert_eq!(projects[0].manager, PackageManager::Pipenv);
        cleanup(&dir);
    }

    #[test]
    fn detect_python_requirements() {
        let dir = make_temp_dir("py_req");
        touch(&dir, "requirements.txt");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Python);
        cleanup(&dir);
    }

    #[test]
    fn detect_python_uv_lock() {
        let dir = make_temp_dir("py_uv");
        touch(&dir, "pyproject.toml");
        touch(&dir, "uv.lock");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].manager, PackageManager::Uv);
        cleanup(&dir);
    }

    #[test]
    fn detect_python_poetry_lock() {
        let dir = make_temp_dir("py_poetry");
        touch(&dir, "pyproject.toml");
        touch(&dir, "poetry.lock");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].manager, PackageManager::Poetry);
        cleanup(&dir);
    }

    #[test]
    fn detect_python_django() {
        let dir = make_temp_dir("py_django");
        write_file(&dir, "requirements.txt", "django==4.2\npsycopg2\n");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Django));
        cleanup(&dir);
    }

    #[test]
    fn detect_python_flask() {
        let dir = make_temp_dir("py_flask");
        write_file(&dir, "requirements.txt", "flask==3.0\n");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Flask));
        cleanup(&dir);
    }

    // --- Ruby ---

    #[test]
    fn detect_ruby_gemfile() {
        let dir = make_temp_dir("rb_gem");
        touch(&dir, "Gemfile");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Ruby);
        assert_eq!(projects[0].manager, PackageManager::Bundler);
        cleanup(&dir);
    }

    #[test]
    fn detect_ruby_with_lock() {
        let dir = make_temp_dir("rb_lock");
        touch(&dir, "Gemfile");
        touch(&dir, "Gemfile.lock");
        let projects = detect(&dir).unwrap();
        assert!(projects[0].lock_file.is_some());
        cleanup(&dir);
    }

    #[test]
    fn detect_ruby_rails() {
        let dir = make_temp_dir("rb_rails");
        touch(&dir, "Gemfile");
        touch(&dir, "config/routes.rb");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Rails));
        cleanup(&dir);
    }

    #[test]
    fn detect_ruby_sinatra() {
        let dir = make_temp_dir("rb_sinatra");
        write_file(&dir, "Gemfile", "gem 'sinatra'\n");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Sinatra));
        cleanup(&dir);
    }

    // --- Rust ---

    #[test]
    fn detect_rust_with_lock() {
        let dir = make_temp_dir("rs_lock");
        touch(&dir, "Cargo.toml");
        touch(&dir, "Cargo.lock");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Rust);
        assert!(projects[0].lock_file.is_some());
        cleanup(&dir);
    }

    #[test]
    fn detect_rust_without_lock() {
        let dir = make_temp_dir("rs_nolock");
        touch(&dir, "Cargo.toml");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Rust);
        assert!(projects[0].lock_file.is_none());
        cleanup(&dir);
    }

    #[test]
    fn detect_rust_axum() {
        let dir = make_temp_dir("rs_axum");
        write_file(&dir, "Cargo.toml", "[dependencies]\naxum = \"0.7\"\n");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Axum));
        cleanup(&dir);
    }

    #[test]
    fn detect_rust_actix() {
        let dir = make_temp_dir("rs_actix");
        write_file(&dir, "Cargo.toml", "[dependencies]\nactix-web = \"4\"\n");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Actix));
        cleanup(&dir);
    }

    // --- Go ---

    #[test]
    fn detect_go() {
        let dir = make_temp_dir("go_basic");
        touch(&dir, "go.mod");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Go);
        cleanup(&dir);
    }

    #[test]
    fn detect_go_with_sum() {
        let dir = make_temp_dir("go_sum");
        touch(&dir, "go.mod");
        touch(&dir, "go.sum");
        let projects = detect(&dir).unwrap();
        assert!(projects[0].lock_file.is_some());
        cleanup(&dir);
    }

    #[test]
    fn detect_go_gin() {
        let dir = make_temp_dir("go_gin");
        write_file(&dir, "go.mod", "require github.com/gin-gonic/gin v1.9.1\n");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Gin));
        cleanup(&dir);
    }

    // --- Java ---

    #[test]
    fn detect_java_maven() {
        let dir = make_temp_dir("java_maven");
        touch(&dir, "pom.xml");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Java);
        assert_eq!(projects[0].manager, PackageManager::Maven);
        cleanup(&dir);
    }

    #[test]
    fn detect_java_gradle() {
        let dir = make_temp_dir("java_gradle");
        touch(&dir, "build.gradle");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].manager, PackageManager::Gradle);
        cleanup(&dir);
    }

    #[test]
    fn detect_java_gradle_kts() {
        let dir = make_temp_dir("java_kts");
        touch(&dir, "build.gradle.kts");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].manager, PackageManager::Gradle);
        cleanup(&dir);
    }

    #[test]
    fn detect_java_gradle_priority_over_maven() {
        let dir = make_temp_dir("java_prio");
        touch(&dir, "build.gradle");
        touch(&dir, "pom.xml");
        let projects = detect(&dir).unwrap();
        // Only one Java project should be detected, and Gradle takes priority
        let java_projects: Vec<_> = projects.iter().filter(|p| p.ecosystem == Ecosystem::Java).collect();
        assert_eq!(java_projects.len(), 1);
        assert_eq!(java_projects[0].manager, PackageManager::Gradle);
        cleanup(&dir);
    }

    #[test]
    fn detect_java_spring() {
        let dir = make_temp_dir("java_spring");
        write_file(&dir, "pom.xml", "<dependency>spring-boot-starter</dependency>\n");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Spring));
        cleanup(&dir);
    }

    // --- .NET ---

    #[test]
    fn detect_dotnet_csproj() {
        let dir = make_temp_dir("dotnet_csproj");
        touch(&dir, "MyApp.csproj");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::DotNet);
        assert_eq!(projects[0].manager, PackageManager::Nuget);
        cleanup(&dir);
    }

    #[test]
    fn detect_dotnet_sln() {
        let dir = make_temp_dir("dotnet_sln");
        touch(&dir, "MyApp.sln");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::DotNet);
        cleanup(&dir);
    }

    #[test]
    fn detect_dotnet_aspnet() {
        let dir = make_temp_dir("dotnet_aspnet");
        write_file(&dir, "MyApp.csproj", "<PackageReference Include=\"Microsoft.AspNetCore.App\" />\n");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::AspNet));
        cleanup(&dir);
    }

    // --- PHP ---

    #[test]
    fn detect_php() {
        let dir = make_temp_dir("php_basic");
        touch(&dir, "composer.json");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Php);
        cleanup(&dir);
    }

    #[test]
    fn detect_php_with_lock() {
        let dir = make_temp_dir("php_lock");
        touch(&dir, "composer.json");
        touch(&dir, "composer.lock");
        let projects = detect(&dir).unwrap();
        assert!(projects[0].lock_file.is_some());
        cleanup(&dir);
    }

    #[test]
    fn detect_php_laravel() {
        let dir = make_temp_dir("php_laravel");
        touch(&dir, "composer.json");
        touch(&dir, "artisan");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Laravel));
        cleanup(&dir);
    }

    // --- Elixir ---

    #[test]
    fn detect_elixir() {
        let dir = make_temp_dir("ex_basic");
        touch(&dir, "mix.exs");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Elixir);
        cleanup(&dir);
    }

    #[test]
    fn detect_elixir_with_lock() {
        let dir = make_temp_dir("ex_lock");
        touch(&dir, "mix.exs");
        touch(&dir, "mix.lock");
        let projects = detect(&dir).unwrap();
        assert!(projects[0].lock_file.is_some());
        cleanup(&dir);
    }

    #[test]
    fn detect_elixir_phoenix() {
        let dir = make_temp_dir("ex_phoenix");
        write_file(&dir, "mix.exs", "{:phoenix, \"~> 1.7\"}\n");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Phoenix));
        cleanup(&dir);
    }

    // --- Swift ---

    #[test]
    fn detect_swift() {
        let dir = make_temp_dir("swift_basic");
        touch(&dir, "Package.swift");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Swift);
        cleanup(&dir);
    }

    #[test]
    fn detect_swift_with_resolved() {
        let dir = make_temp_dir("swift_res");
        touch(&dir, "Package.swift");
        touch(&dir, "Package.resolved");
        let projects = detect(&dir).unwrap();
        assert!(projects[0].lock_file.is_some());
        cleanup(&dir);
    }

    #[test]
    fn detect_swift_vapor() {
        let dir = make_temp_dir("swift_vapor");
        write_file(&dir, "Package.swift", ".package(url: \"https://github.com/vapor/vapor.git\")\n");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Vapor));
        cleanup(&dir);
    }

    // --- Dart ---

    #[test]
    fn detect_dart() {
        let dir = make_temp_dir("dart_basic");
        touch(&dir, "pubspec.yaml");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].ecosystem, Ecosystem::Dart);
        cleanup(&dir);
    }

    #[test]
    fn detect_dart_flutter() {
        let dir = make_temp_dir("dart_flutter");
        touch(&dir, "pubspec.yaml");
        touch(&dir, ".metadata");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects[0].framework, Some(Framework::Flutter));
        cleanup(&dir);
    }

    #[test]
    fn detect_dart_with_lock() {
        let dir = make_temp_dir("dart_lock");
        touch(&dir, "pubspec.yaml");
        touch(&dir, "pubspec.lock");
        let projects = detect(&dir).unwrap();
        assert!(projects[0].lock_file.is_some());
        cleanup(&dir);
    }

    // --- Multi-ecosystem ---

    #[test]
    fn detect_multi_ecosystem() {
        let dir = make_temp_dir("multi");
        touch(&dir, "package.json");
        touch(&dir, "Cargo.toml");
        let projects = detect(&dir).unwrap();
        assert_eq!(projects.len(), 2);
        let ecosystems: Vec<_> = projects.iter().map(|p| &p.ecosystem).collect();
        assert!(ecosystems.contains(&&Ecosystem::Node));
        assert!(ecosystems.contains(&&Ecosystem::Rust));
        cleanup(&dir);
    }
}

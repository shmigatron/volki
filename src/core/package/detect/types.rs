use crate::core::volkiwithstds::io::IoError;
use crate::core::volkiwithstds::path::PathBuf;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ecosystem {
    Node,
    Python,
    Ruby,
    Rust,
    Go,
    Java,
    DotNet,
    Php,
    Elixir,
    Swift,
    Dart,
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ecosystem::Node => write!(f, "Node.js"),
            Ecosystem::Python => write!(f, "Python"),
            Ecosystem::Ruby => write!(f, "Ruby"),
            Ecosystem::Rust => write!(f, "Rust"),
            Ecosystem::Go => write!(f, "Go"),
            Ecosystem::Java => write!(f, "Java"),
            Ecosystem::DotNet => write!(f, ".NET"),
            Ecosystem::Php => write!(f, "PHP"),
            Ecosystem::Elixir => write!(f, "Elixir"),
            Ecosystem::Swift => write!(f, "Swift"),
            Ecosystem::Dart => write!(f, "Dart"),
        }
    }
}

impl Ecosystem {
    pub fn as_toml_str(&self) -> &'static str {
        match self {
            Ecosystem::Node => "node",
            Ecosystem::Python => "python",
            Ecosystem::Ruby => "ruby",
            Ecosystem::Rust => "rust",
            Ecosystem::Go => "go",
            Ecosystem::Java => "java",
            Ecosystem::DotNet => "dotnet",
            Ecosystem::Php => "php",
            Ecosystem::Elixir => "elixir",
            Ecosystem::Swift => "swift",
            Ecosystem::Dart => "dart",
        }
    }

    pub fn from_toml_str(s: &str) -> Option<Self> {
        match s {
            "node" => Some(Ecosystem::Node),
            "python" => Some(Ecosystem::Python),
            "ruby" => Some(Ecosystem::Ruby),
            "rust" => Some(Ecosystem::Rust),
            "go" => Some(Ecosystem::Go),
            "java" => Some(Ecosystem::Java),
            "dotnet" => Some(Ecosystem::DotNet),
            "php" => Some(Ecosystem::Php),
            "elixir" => Some(Ecosystem::Elixir),
            "swift" => Some(Ecosystem::Swift),
            "dart" => Some(Ecosystem::Dart),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Pip,
    Pipenv,
    Poetry,
    Uv,
    Bundler,
    Cargo,
    GoModules,
    Maven,
    Gradle,
    Nuget,
    Composer,
    Mix,
    Spm,
    Pub,
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageManager::Npm => write!(f, "npm"),
            PackageManager::Yarn => write!(f, "yarn"),
            PackageManager::Pnpm => write!(f, "pnpm"),
            PackageManager::Bun => write!(f, "bun"),
            PackageManager::Pip => write!(f, "pip"),
            PackageManager::Pipenv => write!(f, "pipenv"),
            PackageManager::Poetry => write!(f, "poetry"),
            PackageManager::Uv => write!(f, "uv"),
            PackageManager::Bundler => write!(f, "bundler"),
            PackageManager::Cargo => write!(f, "cargo"),
            PackageManager::GoModules => write!(f, "go modules"),
            PackageManager::Maven => write!(f, "maven"),
            PackageManager::Gradle => write!(f, "gradle"),
            PackageManager::Nuget => write!(f, "nuget"),
            PackageManager::Composer => write!(f, "composer"),
            PackageManager::Mix => write!(f, "mix"),
            PackageManager::Spm => write!(f, "spm"),
            PackageManager::Pub => write!(f, "pub"),
        }
    }
}

impl PackageManager {
    pub fn as_toml_str(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Bun => "bun",
            PackageManager::Pip => "pip",
            PackageManager::Pipenv => "pipenv",
            PackageManager::Poetry => "poetry",
            PackageManager::Uv => "uv",
            PackageManager::Bundler => "bundler",
            PackageManager::Cargo => "cargo",
            PackageManager::GoModules => "go_modules",
            PackageManager::Maven => "maven",
            PackageManager::Gradle => "gradle",
            PackageManager::Nuget => "nuget",
            PackageManager::Composer => "composer",
            PackageManager::Mix => "mix",
            PackageManager::Spm => "spm",
            PackageManager::Pub => "pub",
        }
    }

    pub fn from_toml_str(s: &str) -> Option<Self> {
        match s {
            "npm" => Some(PackageManager::Npm),
            "yarn" => Some(PackageManager::Yarn),
            "pnpm" => Some(PackageManager::Pnpm),
            "bun" => Some(PackageManager::Bun),
            "pip" => Some(PackageManager::Pip),
            "pipenv" => Some(PackageManager::Pipenv),
            "poetry" => Some(PackageManager::Poetry),
            "uv" => Some(PackageManager::Uv),
            "bundler" => Some(PackageManager::Bundler),
            "cargo" => Some(PackageManager::Cargo),
            "go_modules" => Some(PackageManager::GoModules),
            "maven" => Some(PackageManager::Maven),
            "gradle" => Some(PackageManager::Gradle),
            "nuget" => Some(PackageManager::Nuget),
            "composer" => Some(PackageManager::Composer),
            "mix" => Some(PackageManager::Mix),
            "spm" => Some(PackageManager::Spm),
            "pub" => Some(PackageManager::Pub),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Framework {
    // Node
    React,
    NextJs,
    Vue,
    Nuxt,
    Angular,
    Svelte,
    SvelteKit,
    Express,
    Fastify,
    Nest,
    Astro,
    Remix,
    Gatsby,
    // Python
    Django,
    Flask,
    FastApi,
    Tornado,
    Pyramid,
    // Ruby
    Rails,
    Sinatra,
    Hanami,
    // Rust
    Actix,
    Axum,
    Rocket,
    Tauri,
    Leptos,
    Yew,
    Bevy,
    // Go
    Gin,
    Echo,
    Fiber,
    Chi,
    Buffalo,
    // Java
    Spring,
    Quarkus,
    Micronaut,
    Jakarta,
    // PHP
    Laravel,
    Symfony,
    Slim,
    CakePhp,
    // .NET
    AspNet,
    Blazor,
    Maui,
    // Elixir
    Phoenix,
    Nerves,
    // Swift
    Vapor,
    SwiftUi,
    // Dart
    Flutter,
    AngularDart,
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Framework::React => write!(f, "React"),
            Framework::NextJs => write!(f, "Next.js"),
            Framework::Vue => write!(f, "Vue"),
            Framework::Nuxt => write!(f, "Nuxt"),
            Framework::Angular => write!(f, "Angular"),
            Framework::Svelte => write!(f, "Svelte"),
            Framework::SvelteKit => write!(f, "SvelteKit"),
            Framework::Express => write!(f, "Express"),
            Framework::Fastify => write!(f, "Fastify"),
            Framework::Nest => write!(f, "NestJS"),
            Framework::Astro => write!(f, "Astro"),
            Framework::Remix => write!(f, "Remix"),
            Framework::Gatsby => write!(f, "Gatsby"),
            Framework::Django => write!(f, "Django"),
            Framework::Flask => write!(f, "Flask"),
            Framework::FastApi => write!(f, "FastAPI"),
            Framework::Tornado => write!(f, "Tornado"),
            Framework::Pyramid => write!(f, "Pyramid"),
            Framework::Rails => write!(f, "Rails"),
            Framework::Sinatra => write!(f, "Sinatra"),
            Framework::Hanami => write!(f, "Hanami"),
            Framework::Actix => write!(f, "Actix Web"),
            Framework::Axum => write!(f, "Axum"),
            Framework::Rocket => write!(f, "Rocket"),
            Framework::Tauri => write!(f, "Tauri"),
            Framework::Leptos => write!(f, "Leptos"),
            Framework::Yew => write!(f, "Yew"),
            Framework::Bevy => write!(f, "Bevy"),
            Framework::Gin => write!(f, "Gin"),
            Framework::Echo => write!(f, "Echo"),
            Framework::Fiber => write!(f, "Fiber"),
            Framework::Chi => write!(f, "Chi"),
            Framework::Buffalo => write!(f, "Buffalo"),
            Framework::Spring => write!(f, "Spring Boot"),
            Framework::Quarkus => write!(f, "Quarkus"),
            Framework::Micronaut => write!(f, "Micronaut"),
            Framework::Jakarta => write!(f, "Jakarta EE"),
            Framework::Laravel => write!(f, "Laravel"),
            Framework::Symfony => write!(f, "Symfony"),
            Framework::Slim => write!(f, "Slim"),
            Framework::CakePhp => write!(f, "CakePHP"),
            Framework::AspNet => write!(f, "ASP.NET"),
            Framework::Blazor => write!(f, "Blazor"),
            Framework::Maui => write!(f, "MAUI"),
            Framework::Phoenix => write!(f, "Phoenix"),
            Framework::Nerves => write!(f, "Nerves"),
            Framework::Vapor => write!(f, "Vapor"),
            Framework::SwiftUi => write!(f, "SwiftUI"),
            Framework::Flutter => write!(f, "Flutter"),
            Framework::AngularDart => write!(f, "AngularDart"),
        }
    }
}

impl Framework {
    pub fn as_toml_str(&self) -> &'static str {
        match self {
            Framework::React => "react",
            Framework::NextJs => "nextjs",
            Framework::Vue => "vue",
            Framework::Nuxt => "nuxt",
            Framework::Angular => "angular",
            Framework::Svelte => "svelte",
            Framework::SvelteKit => "sveltekit",
            Framework::Express => "express",
            Framework::Fastify => "fastify",
            Framework::Nest => "nest",
            Framework::Astro => "astro",
            Framework::Remix => "remix",
            Framework::Gatsby => "gatsby",
            Framework::Django => "django",
            Framework::Flask => "flask",
            Framework::FastApi => "fastapi",
            Framework::Tornado => "tornado",
            Framework::Pyramid => "pyramid",
            Framework::Rails => "rails",
            Framework::Sinatra => "sinatra",
            Framework::Hanami => "hanami",
            Framework::Actix => "actix",
            Framework::Axum => "axum",
            Framework::Rocket => "rocket",
            Framework::Tauri => "tauri",
            Framework::Leptos => "leptos",
            Framework::Yew => "yew",
            Framework::Bevy => "bevy",
            Framework::Gin => "gin",
            Framework::Echo => "echo",
            Framework::Fiber => "fiber",
            Framework::Chi => "chi",
            Framework::Buffalo => "buffalo",
            Framework::Spring => "spring",
            Framework::Quarkus => "quarkus",
            Framework::Micronaut => "micronaut",
            Framework::Jakarta => "jakarta",
            Framework::Laravel => "laravel",
            Framework::Symfony => "symfony",
            Framework::Slim => "slim",
            Framework::CakePhp => "cakephp",
            Framework::AspNet => "aspnet",
            Framework::Blazor => "blazor",
            Framework::Maui => "maui",
            Framework::Phoenix => "phoenix",
            Framework::Nerves => "nerves",
            Framework::Vapor => "vapor",
            Framework::SwiftUi => "swiftui",
            Framework::Flutter => "flutter",
            Framework::AngularDart => "angulardart",
        }
    }

    pub fn from_toml_str(s: &str) -> Option<Self> {
        match s {
            "react" => Some(Framework::React),
            "nextjs" => Some(Framework::NextJs),
            "vue" => Some(Framework::Vue),
            "nuxt" => Some(Framework::Nuxt),
            "angular" => Some(Framework::Angular),
            "svelte" => Some(Framework::Svelte),
            "sveltekit" => Some(Framework::SvelteKit),
            "express" => Some(Framework::Express),
            "fastify" => Some(Framework::Fastify),
            "nest" => Some(Framework::Nest),
            "astro" => Some(Framework::Astro),
            "remix" => Some(Framework::Remix),
            "gatsby" => Some(Framework::Gatsby),
            "django" => Some(Framework::Django),
            "flask" => Some(Framework::Flask),
            "fastapi" => Some(Framework::FastApi),
            "tornado" => Some(Framework::Tornado),
            "pyramid" => Some(Framework::Pyramid),
            "rails" => Some(Framework::Rails),
            "sinatra" => Some(Framework::Sinatra),
            "hanami" => Some(Framework::Hanami),
            "actix" => Some(Framework::Actix),
            "axum" => Some(Framework::Axum),
            "rocket" => Some(Framework::Rocket),
            "tauri" => Some(Framework::Tauri),
            "leptos" => Some(Framework::Leptos),
            "yew" => Some(Framework::Yew),
            "bevy" => Some(Framework::Bevy),
            "gin" => Some(Framework::Gin),
            "echo" => Some(Framework::Echo),
            "fiber" => Some(Framework::Fiber),
            "chi" => Some(Framework::Chi),
            "buffalo" => Some(Framework::Buffalo),
            "spring" => Some(Framework::Spring),
            "quarkus" => Some(Framework::Quarkus),
            "micronaut" => Some(Framework::Micronaut),
            "jakarta" => Some(Framework::Jakarta),
            "laravel" => Some(Framework::Laravel),
            "symfony" => Some(Framework::Symfony),
            "slim" => Some(Framework::Slim),
            "cakephp" => Some(Framework::CakePhp),
            "aspnet" => Some(Framework::AspNet),
            "blazor" => Some(Framework::Blazor),
            "maui" => Some(Framework::Maui),
            "phoenix" => Some(Framework::Phoenix),
            "nerves" => Some(Framework::Nerves),
            "vapor" => Some(Framework::Vapor),
            "swiftui" => Some(Framework::SwiftUi),
            "flutter" => Some(Framework::Flutter),
            "angulardart" => Some(Framework::AngularDart),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectedProject {
    pub ecosystem: Ecosystem,
    pub manager: PackageManager,
    pub manifest: PathBuf,
    pub lock_file: Option<PathBuf>,
    pub framework: Option<Framework>,
}

#[derive(Debug)]
pub enum DetectError {
    Io(IoError),
    NotADirectory(PathBuf),
}

impl fmt::Display for DetectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectError::Io(err) => write!(f, "I/O error: {err}"),
            DetectError::NotADirectory(path) => write!(f, "not a directory: {}", path.as_str()),
        }
    }
}

impl From<IoError> for DetectError {
    fn from(err: IoError) -> Self {
        DetectError::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::volkiwithstds::io::IoErrorKind;

    #[test]
    fn ecosystem_display() {
        assert_eq!(crate::vformat!("{}", Ecosystem::Node).as_str(), "Node.js");
        assert_eq!(crate::vformat!("{}", Ecosystem::Python).as_str(), "Python");
        assert_eq!(crate::vformat!("{}", Ecosystem::Ruby).as_str(), "Ruby");
        assert_eq!(crate::vformat!("{}", Ecosystem::Rust).as_str(), "Rust");
        assert_eq!(crate::vformat!("{}", Ecosystem::Go).as_str(), "Go");
        assert_eq!(crate::vformat!("{}", Ecosystem::Java).as_str(), "Java");
        assert_eq!(crate::vformat!("{}", Ecosystem::DotNet).as_str(), ".NET");
        assert_eq!(crate::vformat!("{}", Ecosystem::Php).as_str(), "PHP");
        assert_eq!(crate::vformat!("{}", Ecosystem::Elixir).as_str(), "Elixir");
        assert_eq!(crate::vformat!("{}", Ecosystem::Swift).as_str(), "Swift");
        assert_eq!(crate::vformat!("{}", Ecosystem::Dart).as_str(), "Dart");
    }

    #[test]
    fn ecosystem_toml_roundtrip() {
        let ecosystems = [
            Ecosystem::Node,
            Ecosystem::Python,
            Ecosystem::Ruby,
            Ecosystem::Rust,
            Ecosystem::Go,
            Ecosystem::Java,
            Ecosystem::DotNet,
            Ecosystem::Php,
            Ecosystem::Elixir,
            Ecosystem::Swift,
            Ecosystem::Dart,
        ];
        for eco in &ecosystems {
            let s = eco.as_toml_str();
            assert_eq!(Ecosystem::from_toml_str(s).as_ref(), Some(eco));
        }
    }

    #[test]
    fn ecosystem_from_toml_unknown() {
        assert_eq!(Ecosystem::from_toml_str("unknown"), None);
    }

    #[test]
    fn package_manager_display() {
        assert_eq!(crate::vformat!("{}", PackageManager::Npm).as_str(), "npm");
        assert_eq!(crate::vformat!("{}", PackageManager::Yarn).as_str(), "yarn");
        assert_eq!(crate::vformat!("{}", PackageManager::Pnpm).as_str(), "pnpm");
        assert_eq!(crate::vformat!("{}", PackageManager::Bun).as_str(), "bun");
        assert_eq!(crate::vformat!("{}", PackageManager::Pip).as_str(), "pip");
        assert_eq!(
            crate::vformat!("{}", PackageManager::Pipenv).as_str(),
            "pipenv"
        );
        assert_eq!(
            crate::vformat!("{}", PackageManager::Poetry).as_str(),
            "poetry"
        );
        assert_eq!(crate::vformat!("{}", PackageManager::Uv).as_str(), "uv");
        assert_eq!(
            crate::vformat!("{}", PackageManager::Bundler).as_str(),
            "bundler"
        );
        assert_eq!(
            crate::vformat!("{}", PackageManager::Cargo).as_str(),
            "cargo"
        );
        assert_eq!(
            crate::vformat!("{}", PackageManager::GoModules).as_str(),
            "go modules"
        );
        assert_eq!(
            crate::vformat!("{}", PackageManager::Maven).as_str(),
            "maven"
        );
        assert_eq!(
            crate::vformat!("{}", PackageManager::Gradle).as_str(),
            "gradle"
        );
        assert_eq!(
            crate::vformat!("{}", PackageManager::Nuget).as_str(),
            "nuget"
        );
        assert_eq!(
            crate::vformat!("{}", PackageManager::Composer).as_str(),
            "composer"
        );
        assert_eq!(crate::vformat!("{}", PackageManager::Mix).as_str(), "mix");
        assert_eq!(crate::vformat!("{}", PackageManager::Spm).as_str(), "spm");
        assert_eq!(crate::vformat!("{}", PackageManager::Pub).as_str(), "pub");
    }

    #[test]
    fn package_manager_toml_roundtrip() {
        let managers = [
            PackageManager::Npm,
            PackageManager::Yarn,
            PackageManager::Pnpm,
            PackageManager::Bun,
            PackageManager::Pip,
            PackageManager::Pipenv,
            PackageManager::Poetry,
            PackageManager::Uv,
            PackageManager::Bundler,
            PackageManager::Cargo,
            PackageManager::GoModules,
            PackageManager::Maven,
            PackageManager::Gradle,
            PackageManager::Nuget,
            PackageManager::Composer,
            PackageManager::Mix,
            PackageManager::Spm,
            PackageManager::Pub,
        ];
        for mgr in &managers {
            let s = mgr.as_toml_str();
            assert_eq!(PackageManager::from_toml_str(s).as_ref(), Some(mgr));
        }
    }

    #[test]
    fn package_manager_from_toml_unknown() {
        assert_eq!(PackageManager::from_toml_str("unknown"), None);
    }

    #[test]
    fn framework_display() {
        assert_eq!(crate::vformat!("{}", Framework::React).as_str(), "React");
        assert_eq!(crate::vformat!("{}", Framework::NextJs).as_str(), "Next.js");
        assert_eq!(crate::vformat!("{}", Framework::Rails).as_str(), "Rails");
        assert_eq!(
            crate::vformat!("{}", Framework::Flutter).as_str(),
            "Flutter"
        );
        assert_eq!(
            crate::vformat!("{}", Framework::Spring).as_str(),
            "Spring Boot"
        );
        assert_eq!(
            crate::vformat!("{}", Framework::FastApi).as_str(),
            "FastAPI"
        );
        assert_eq!(crate::vformat!("{}", Framework::AspNet).as_str(), "ASP.NET");
    }

    #[test]
    fn framework_toml_roundtrip() {
        let frameworks = [
            Framework::React,
            Framework::NextJs,
            Framework::Vue,
            Framework::Nuxt,
            Framework::Angular,
            Framework::Svelte,
            Framework::SvelteKit,
            Framework::Express,
            Framework::Fastify,
            Framework::Nest,
            Framework::Astro,
            Framework::Remix,
            Framework::Gatsby,
            Framework::Django,
            Framework::Flask,
            Framework::FastApi,
            Framework::Tornado,
            Framework::Pyramid,
            Framework::Rails,
            Framework::Sinatra,
            Framework::Hanami,
            Framework::Actix,
            Framework::Axum,
            Framework::Rocket,
            Framework::Tauri,
            Framework::Leptos,
            Framework::Yew,
            Framework::Bevy,
            Framework::Gin,
            Framework::Echo,
            Framework::Fiber,
            Framework::Chi,
            Framework::Buffalo,
            Framework::Spring,
            Framework::Quarkus,
            Framework::Micronaut,
            Framework::Jakarta,
            Framework::Laravel,
            Framework::Symfony,
            Framework::Slim,
            Framework::CakePhp,
            Framework::AspNet,
            Framework::Blazor,
            Framework::Maui,
            Framework::Phoenix,
            Framework::Nerves,
            Framework::Vapor,
            Framework::SwiftUi,
            Framework::Flutter,
            Framework::AngularDart,
        ];
        for fw in &frameworks {
            let s = fw.as_toml_str();
            assert_eq!(Framework::from_toml_str(s).as_ref(), Some(fw));
        }
    }

    #[test]
    fn framework_from_toml_unknown() {
        assert_eq!(Framework::from_toml_str("unknown"), None);
    }

    #[test]
    fn detect_error_display_io() {
        let err = DetectError::Io(IoError::new(IoErrorKind::NotFound, "not found"));
        assert!(crate::vformat!("{err}").contains("I/O error"));
    }

    #[test]
    fn detect_error_display_not_a_directory() {
        let err = DetectError::NotADirectory(PathBuf::from("/tmp/foo"));
        let msg = crate::vformat!("{err}");
        assert!(msg.contains("not a directory"));
        assert!(msg.contains("/tmp/foo"));
    }

    #[test]
    fn detect_error_from_io() {
        let io_err = IoError::new(IoErrorKind::PermissionDenied, "denied");
        let err: DetectError = io_err.into();
        assert!(matches!(err, DetectError::Io(_)));
    }
}

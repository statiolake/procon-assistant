// if_chain! が clippy::collapsible_match に引っかかるようになっているが、それを阻止したい。
#![allow(clippy::collapsible_match)]

use super::{FilesToOpen, Preprocessed, RawSource};
use super::{Lang, Progress};
use crate::eprintln_debug;
use crate::imp::config::MinifyMode;
use crate::imp::config::RustProjectTemplate;
use crate::imp::config::CONFIG;
use crate::imp::process;
use anyhow::{anyhow, bail, ensure};
use anyhow::{Context, Result};
use fs_extra::dir;
use fs_extra::dir::CopyOptions;
use if_chain::if_chain;
use itertools::Itertools;
use lazy_static::lazy_static;
use quote::ToTokens;
use regex::Regex;
use scopefunc::ScopeFunc;
use std::fs as stdfs;
use std::io::prelude::*;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::{Command, Stdio};

pub struct Rust2020;
pub struct Rust2016;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RustVersion {
    Rust2020,
    Rust2016,
}

lazy_static! {
    static ref RE_MOD_PATH: Regex = Regex::new(r#"#\[path\s+=\s+"(?P<path>.*)"\]"#).unwrap();
    static ref RE_MOD: Regex = Regex::new(r#"\bmod\s+(?P<name>\w+);"#).unwrap();
    static ref RE_COMMENT: Regex = Regex::new(r#"//.*"#).unwrap();
    static ref RE_WHITESPACE_AFTER_COLONS: Regex = Regex::new(r#"\s*(?P<col>[;:])\s*"#).unwrap();
    static ref RE_MULTIPLE_SPACE: Regex = Regex::new(r#"\s+"#).unwrap();
    static ref RE_WHITESPACE_AROUND_OPERATOR: Regex =
        Regex::new(r#"\s*(?P<op>[+\-*/%~^|&<>=,.!?\[\]]|<<|>>|<=|>=|==|!=|\+=|-=|\*=|/=|->)\s*"#)
            .unwrap();
    static ref RE_WHITESPACE_AROUND_PAREN: Regex = Regex::new(r#"\s*(?P<par>[({)}])\s*"#).unwrap();
}

impl Lang for Rust2020 {
    fn check() -> Result<bool>
    where
        Self: Sized,
    {
        check(RustVersion::Rust2020)
    }

    fn new_boxed() -> Result<Box<dyn Lang>>
    where
        Self: Sized,
    {
        Ok(Box::new(Rust2020))
    }

    fn lang_name() -> &'static str
    where
        Self: Sized,
    {
        "rust2020"
    }

    fn init_async(&self) -> Progress<Result<()>> {
        init_async(RustVersion::Rust2020)
    }

    fn to_open(&self) -> Result<FilesToOpen> {
        Ok(to_open(RustVersion::Rust2020))
    }

    fn open_docs(&self) -> Result<()> {
        open_docs(RustVersion::Rust2020)
    }

    fn get_source(&self) -> Result<RawSource> {
        get_source(RustVersion::Rust2020)
    }

    fn needs_compile(&self) -> Result<bool> {
        Ok(needs_compile(RustVersion::Rust2020, false))
    }

    fn needs_release_compile(&self) -> Result<bool> {
        Ok(needs_compile(RustVersion::Rust2020, true))
    }

    fn compile_command(&self) -> Result<Vec<Command>> {
        compile_command(RustVersion::Rust2020, false)
    }

    fn release_compile_command(&self) -> Result<Vec<Command>> {
        compile_command(RustVersion::Rust2020, true)
    }

    fn run_command(&self) -> Result<Command> {
        run_command(RustVersion::Rust2020, false)
    }

    fn release_run_command(&self) -> Result<Command> {
        run_command(RustVersion::Rust2020, true)
    }

    fn preprocess(&self, source: &RawSource, minify: MinifyMode) -> Result<Preprocessed> {
        preprocess(RustVersion::Rust2020, source, minify)
    }

    fn lint(&self, source: &RawSource) -> Result<Vec<String>> {
        lint(RustVersion::Rust2020, source)
    }
}

impl Lang for Rust2016 {
    fn check() -> Result<bool>
    where
        Self: Sized,
    {
        check(RustVersion::Rust2016)
    }

    fn new_boxed() -> Result<Box<dyn Lang>>
    where
        Self: Sized,
    {
        Ok(Box::new(Rust2016))
    }

    fn lang_name() -> &'static str
    where
        Self: Sized,
    {
        "rust2016"
    }

    fn init_async(&self) -> Progress<anyhow::Result<()>> {
        init_async(RustVersion::Rust2016)
    }

    fn to_open(&self) -> Result<FilesToOpen> {
        Ok(to_open(RustVersion::Rust2016))
    }

    fn open_docs(&self) -> Result<()> {
        open_docs(RustVersion::Rust2016)
    }

    fn get_source(&self) -> Result<RawSource> {
        get_source(RustVersion::Rust2016)
    }

    fn needs_compile(&self) -> Result<bool> {
        Ok(needs_compile(RustVersion::Rust2016, false))
    }

    fn needs_release_compile(&self) -> Result<bool> {
        Ok(needs_compile(RustVersion::Rust2016, true))
    }

    fn compile_command(&self) -> Result<Vec<Command>> {
        compile_command(RustVersion::Rust2016, false)
    }

    fn release_compile_command(&self) -> Result<Vec<Command>> {
        compile_command(RustVersion::Rust2016, true)
    }

    fn run_command(&self) -> Result<Command> {
        run_command(RustVersion::Rust2016, false)
    }

    fn release_run_command(&self) -> Result<Command> {
        run_command(RustVersion::Rust2016, true)
    }

    fn preprocess(&self, source: &RawSource, minify: MinifyMode) -> Result<Preprocessed> {
        preprocess(RustVersion::Rust2016, source, minify)
    }

    fn lint(&self, source: &RawSource) -> Result<Vec<String>> {
        lint(RustVersion::Rust2016, source)
    }
}

fn check(ver: RustVersion) -> Result<bool> {
    let path_cargo_toml = Path::new("main/Cargo.toml");
    // check the existance of Cargo.toml first. if not, that's not a Rust project.
    if !path_cargo_toml.exists() {
        return Ok(false);
    }

    // read Cargo.toml and check `edition` to determine Rust version
    use toml::Value;
    let cargo_toml: Value = stdfs::read_to_string(path_cargo_toml)
        .context("failed to read Cargo.toml")?
        .parse()
        .context("failed to parse Cargo.toml")?;
    let guessed_ver = cargo_toml
        .get("package")
        .and_then(|package| package.as_table())
        .ok_or_else(|| anyhow!("malformed Cargo.toml: no [package] section"))?
        .get("edition")
        .map(|edition| match edition.as_str() {
            None => Ok(RustVersion::Rust2016),
            Some("2015") => Ok(RustVersion::Rust2016),
            Some("2018") => Ok(RustVersion::Rust2020),
            Some(edition) => bail!("failed to guess the version: unknown edition {}", edition),
        })
        .unwrap_or(Ok(RustVersion::Rust2016))?;
    Ok(ver == guessed_ver)
}

fn init_async(ver: RustVersion) -> Progress<Result<()>> {
    Progress::from_fn(move |sender| {
        let _ = sender.send("generating new cargo project".into());
        // generate a project
        match ver {
            RustVersion::Rust2020 => {
                match &CONFIG.langs.rust2020.project_template {
                    RustProjectTemplate::Git { repository, branch } => {
                        generate_git(repository, branch)
                    }
                    RustProjectTemplate::Local { path } => generate_local(path),
                }
                .context("failed to generate a project")?;
            }
            RustVersion::Rust2016 => {
                let path = CONFIG
                    .langs
                    .rust2016
                    .project_template_path
                    .as_ref()
                    .ok_or_else(|| anyhow!("project template for Rust 2016 is not specified"))?;
                generate_local(path).context("failed to generate a project")?;
            }
        }

        let _ = sender.send("building generated project".into());
        // pre-build the project
        let output = Command::new("cargo")
            .arg("build")
            .arg("--quiet")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .current_dir("main")
            .spawn()?
            .wait_with_output()?;
        ensure!(
            output.status.success(),
            "failed to build a project: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        Ok(())
    })
}

fn to_open(_ver: RustVersion) -> FilesToOpen {
    FilesToOpen {
        files: vec![Path::new("main").join("src").join("main.rs")],
        directory: PathBuf::from("main"),
    }
}

fn open_docs(_ver: RustVersion) -> Result<()> {
    // open crate docs
    let path = to_absolute::to_absolute_from_current_dir("main/target/doc/main/index.html")
        .context("failed to get the absolute path for the document")?;
    let path_url_base = path.display().to_string().replace(MAIN_SEPARATOR, "/");
    let crate_docs = format!("file:///{}", path_url_base);
    process::open_browser(&crate_docs).context("failed to open crate docs")?;

    // open std docs
    Command::new("rustup")
        .arg("doc")
        .arg("--std")
        .spawn()
        .context("failed to open std doc")?;

    Ok(())
}

fn get_source(_ver: RustVersion) -> Result<RawSource> {
    stdfs::read_to_string("main/src/main.rs")
        .map(RawSource)
        .map_err(Into::into)
}

fn needs_compile(_ver: RustVersion, _release: bool) -> bool {
    // in Rust, to avoid copying a large `target` directory, `target`
    // directory is symlinked to the template directory. This means that the
    // binary is placed in the same place for all projects. It causes the
    // binary overwritten by another project. To prevent running wrong
    // binary, we always need to clean the binary and compile.
    true
}

fn compile_command(ver: RustVersion, release: bool) -> Result<Vec<Command>> {
    let cargo = which::which("cargo").map_err(|_| anyhow!("failed to find cargo in your PATH"))?;
    let ver = match ver {
        RustVersion::Rust2020 => "+1.42.0",
        RustVersion::Rust2016 => "+1.15.0",
    };

    let clean = Command::new(&cargo).modify(|cmd| {
        cmd.arg(ver).arg("clean");
        if release {
            cmd.arg("--release");
        }
        cmd.arg("-p").arg("main").current_dir("main");
    });

    let build = Command::new(&cargo).modify(|cmd| {
        cmd.arg(ver).arg("build");
        if release {
            cmd.arg("--release");
        }
        cmd.current_dir("main");
    });

    Ok(vec![clean, build])
}

fn run_command(_ver: RustVersion, release: bool) -> Result<Command> {
    let target = if release { "release" } else { "debug" };
    let binary = which::which(format!("main/target/{}/main", target))
        .map_err(|_| anyhow!("failed to get the built binary"))?;

    Ok(Command::new(binary).modify(|cmd| {
        cmd.env("RUST_BACKTRACE", "1");
    }))
}

fn preprocess(
    ver: RustVersion,
    RawSource(source): &RawSource,
    minify: MinifyMode,
) -> Result<Preprocessed> {
    let source = expand_source(ver, Path::new("main/src"), &source, minify)?;
    Ok(Preprocessed(source))
}

fn lint(ver: RustVersion, source: &RawSource) -> Result<Vec<String>> {
    let Preprocessed(pped) =
        preprocess(ver, source, MinifyMode::All).context("failed to preprocess the source")?;

    let mut res = Vec::new();
    if pped.contains("eprintln!") {
        res.push("eprintln! found".to_string());
    }
    if pped.contains("dbg!") {
        res.push("dbg! found".to_string());
    }

    Ok(res)
}

fn generate_git(repository: &str, branch: &str) -> Result<()> {
    if Path::new("main").exists() {
        // skip generating everything if main directory exists
        return Ok(());
    }

    if which::which("cargo-generate").is_err() {
        bail!(concat!(
            "generating a project from git requires cargo-generate to be installed; ",
            "if you haven't, type 'cargo install cargo-generate'."
        ));
    }

    let output = Command::new("cargo")
        .arg("generate")
        .arg("--git")
        .arg(repository)
        .arg("--branch")
        .arg(branch)
        .arg("--name")
        .arg("main")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    ensure!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

fn generate_local(path: &Path) -> Result<()> {
    eprintln_debug!("copying from `{}`", path.display());

    let options = CopyOptions {
        skip_exist: true,
        copy_inside: true,
        ..CopyOptions::new()
    };

    let base_path = Path::new("main");
    stdfs::create_dir_all(base_path)?;
    for entry in stdfs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry_path
            .file_name()
            .expect("critical error: file has no name");
        let target = base_path.join(entry_name);
        if target.exists() {
            // skip existing file or folder
            continue;
        }

        let entry_metadata = entry.metadata()?;
        if entry_metadata.is_file() {
            stdfs::copy(&entry_path, &target)?;
        } else if entry_metadata.is_dir() && entry_name != "target" {
            dir::copy(&entry_path, &target, &options)?;
        }
    }

    // symlink target directory
    let template_target_path = path.join("target");
    let project_target_path = base_path.join("target");
    if template_target_path.exists() && !project_target_path.exists() {
        symlink::symlink_dir(template_target_path, project_target_path)?;
    }

    Ok(())
}

fn expand_source(ver: RustVersion, cwd: &Path, source: &str, mode: MinifyMode) -> Result<String> {
    let file = syn::parse_file(&source).context("failed to parse the source code")?;
    let mut file = expand_file(cwd, file)?;
    remove_doc_comments(&mut file);
    remove_tests(&mut file);
    remove_cfg_version(ver, &mut file)?;

    match mode {
        MinifyMode::None => rustfmt(&file.into_token_stream().to_string()),
        MinifyMode::All => minify(&file.into_token_stream().to_string()),
        MinifyMode::TemplateOnly => {
            let syn::File {
                shebang,
                attrs,
                items,
            } = file;

            let mut res = String::new();
            if let Some(shebang) = shebang {
                res.push_str(&shebang);
                res.push('\n');
            }

            res.push_str(&minify(
                &attrs
                    .into_iter()
                    .map(|attr| attr.into_token_stream())
                    .collect::<proc_macro2::TokenStream>()
                    .to_string(),
            )?);
            for item in items {
                match item {
                    syn::Item::Mod(imod) => {
                        res.push_str(&minify(&imod.into_token_stream().to_string())?);
                        res.push('\n');
                    }
                    other => res.push_str(&rustfmt(&other.into_token_stream().to_string())?),
                }
            }

            Ok(res)
        }
    }
}

fn expand_file(cwd: &Path, mut file: syn::File) -> Result<syn::File> {
    file.items = file
        .items
        .into_iter()
        .map(|item| match item {
            syn::Item::Mod(imod) => expand_mod(cwd, imod).map(Into::into),
            item => Ok(item),
        })
        .collect::<Result<_>>()?;

    Ok(file)
}

fn expand_mod(cwd: &Path, imod: syn::ItemMod) -> Result<syn::ItemMod> {
    let semi_span = match imod.semi {
        Some(semi) => semi.spans[0],
        None => return Ok(imod),
    };

    let attrs = imod.attrs;
    let mut paths = Vec::new();
    let mut rest_attrs = Vec::new();
    for attr in attrs {
        let meta = match attr.parse_meta() {
            Ok(meta) => meta,
            Err(_) => {
                rest_attrs.push(attr);
                continue;
            }
        };

        match meta {
            syn::Meta::NameValue(syn::MetaNameValue { path, lit, .. }) if matches!(path.get_ident(), Some(ident) if ident == "path") => {
                paths.push(lit)
            }
            _ => {
                rest_attrs.push(attr);
                continue;
            }
        }
    }

    ensure!(paths.len() <= 1, "multiple paths are specified for module");
    let (path, next_cwd) = match paths.into_iter().next() {
        Some(path) => match path {
            syn::Lit::Str(s) => {
                let path = PathBuf::from(s.value());
                let next_cwd = path
                    .parent()
                    .expect("failed to get parent directory")
                    .to_path_buf();
                (path, next_cwd)
            }
            _ => bail!("invalid value of type for `#[path = ]`"),
        },
        None => {
            let dir = PathBuf::from(imod.ident.to_string());
            let file = PathBuf::from(format!("{}.rs", imod.ident));
            let dirmod = dir.join("mod.rs");
            eprintln_debug!("searching file: {}", file.display());
            eprintln_debug!("searching dir: {}", dir.display());

            if cwd.join(&file).exists() {
                (file, cwd.join(dir))
            } else if cwd.join(&dirmod).exists() {
                (dirmod, cwd.join(dir))
            } else {
                bail!("failed to find the module");
            }
        }
    };

    // load file from the path and parse
    let source = stdfs::read_to_string(&cwd.join(&path)).with_context(|| {
        format!(
            "failed to read next file `{}` in `{}`",
            path.display(),
            cwd.display()
        )
    })?;
    let file = syn::parse_file(&source).context("failed to parse next module file")?;
    let expanded = expand_file(&next_cwd, file).context("failed to expand next module file")?;

    rest_attrs.extend(expanded.attrs);
    Ok(syn::ItemMod {
        attrs: rest_attrs,
        vis: imod.vis,
        mod_token: imod.mod_token,
        ident: imod.ident,
        content: Some((syn::token::Brace { span: semi_span }, expanded.items)),
        semi: None,
    })
}

fn remove_doc_comments(file: &mut syn::File) {
    use syn::visit_mut;
    use syn::visit_mut::VisitMut;
    use syn::*;

    RemoveDocCommentsVisitor.visit_file_mut(file);

    struct RemoveDocCommentsVisitor;
    impl VisitMut for RemoveDocCommentsVisitor {
        fn visit_field_mut(&mut self, node: &mut Field) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_field_mut(self, node);
        }

        fn visit_file_mut(&mut self, node: &mut File) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_file_mut(self, node);
        }

        fn visit_foreign_item_fn_mut(&mut self, node: &mut ForeignItemFn) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_foreign_item_fn_mut(self, node);
        }

        fn visit_foreign_item_macro_mut(&mut self, node: &mut ForeignItemMacro) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_foreign_item_macro_mut(self, node);
        }

        fn visit_foreign_item_static_mut(&mut self, node: &mut ForeignItemStatic) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_foreign_item_static_mut(self, node);
        }

        fn visit_foreign_item_type_mut(&mut self, node: &mut ForeignItemType) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_foreign_item_type_mut(self, node);
        }

        fn visit_impl_item_const_mut(&mut self, node: &mut ImplItemConst) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_impl_item_const_mut(self, node);
        }

        fn visit_impl_item_macro_mut(&mut self, node: &mut ImplItemMacro) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_impl_item_macro_mut(self, node);
        }

        fn visit_impl_item_method_mut(&mut self, node: &mut ImplItemMethod) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_impl_item_method_mut(self, node);
        }

        fn visit_impl_item_type_mut(&mut self, node: &mut ImplItemType) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_impl_item_type_mut(self, node);
        }

        fn visit_item_const_mut(&mut self, node: &mut ItemConst) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_const_mut(self, node);
        }

        fn visit_item_enum_mut(&mut self, node: &mut ItemEnum) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_enum_mut(self, node);
        }

        fn visit_item_extern_crate_mut(&mut self, node: &mut ItemExternCrate) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_extern_crate_mut(self, node);
        }

        fn visit_item_fn_mut(&mut self, node: &mut ItemFn) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_fn_mut(self, node);
        }

        fn visit_item_foreign_mod_mut(&mut self, node: &mut ItemForeignMod) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_foreign_mod_mut(self, node);
        }

        fn visit_item_impl_mut(&mut self, node: &mut ItemImpl) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_impl_mut(self, node);
        }

        fn visit_item_macro_mut(&mut self, node: &mut ItemMacro) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_macro_mut(self, node);
        }

        fn visit_item_macro2_mut(&mut self, node: &mut ItemMacro2) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_macro2_mut(self, node);
        }

        fn visit_item_static_mut(&mut self, node: &mut ItemStatic) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_static_mut(self, node);
        }

        fn visit_item_struct_mut(&mut self, node: &mut ItemStruct) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_struct_mut(self, node);
        }

        fn visit_item_trait_mut(&mut self, node: &mut ItemTrait) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_trait_mut(self, node);
        }

        fn visit_item_trait_alias_mut(&mut self, node: &mut ItemTraitAlias) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_trait_alias_mut(self, node);
        }

        fn visit_item_type_mut(&mut self, node: &mut ItemType) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_type_mut(self, node);
        }

        fn visit_item_union_mut(&mut self, node: &mut ItemUnion) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_union_mut(self, node);
        }

        fn visit_item_use_mut(&mut self, node: &mut ItemUse) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_use_mut(self, node);
        }

        fn visit_lifetime_def_mut(&mut self, node: &mut LifetimeDef) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_lifetime_def_mut(self, node);
        }

        fn visit_trait_item_const_mut(&mut self, node: &mut TraitItemConst) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_trait_item_const_mut(self, node);
        }

        fn visit_trait_item_macro_mut(&mut self, node: &mut TraitItemMacro) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_trait_item_macro_mut(self, node);
        }

        fn visit_trait_item_method_mut(&mut self, node: &mut TraitItemMethod) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_trait_item_method_mut(self, node);
        }

        fn visit_trait_item_type_mut(&mut self, node: &mut TraitItemType) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_trait_item_type_mut(self, node);
        }

        fn visit_type_param_mut(&mut self, node: &mut TypeParam) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_type_param_mut(self, node);
        }

        fn visit_variant_mut(&mut self, node: &mut Variant) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_variant_mut(self, node);
        }

        fn visit_item_mod_mut(&mut self, node: &mut ItemMod) {
            remove_doc_attr(&mut node.attrs);
            visit_mut::visit_item_mod_mut(self, node);
        }
    }

    fn remove_doc_attr(attrs: &mut Vec<Attribute>) {
        attrs.retain(|attr| {
            if_chain! {
                if let Ok(meta) = attr.parse_meta();
                if let Meta::NameValue(meta) = meta;
                if let Some(ident) = meta.path.get_ident();
                then {
                    ident != "doc"
                } else {
                    true
                }
            }
        });
    }
}

fn remove_tests(file: &mut syn::File) {
    ItemRemover.visit_file_mut(file);

    use syn::visit_mut;
    use syn::visit_mut::VisitMut;
    use syn::*;

    struct ItemRemover;
    impl VisitMut for ItemRemover {
        fn visit_file_mut(&mut self, node: &mut File) {
            node.items.retain(|item| retains_item(item));
            visit_mut::visit_file_mut(self, node);
        }

        fn visit_item_mod_mut(&mut self, node: &mut ItemMod) {
            if let Some((_, items)) = &mut node.content {
                items.retain(|item| retains_item(item));
            }

            visit_mut::visit_item_mod_mut(self, node);
        }

        fn visit_block_mut(&mut self, node: &mut Block) {
            node.stmts.retain(|stmt| {
                if let Stmt::Item(item) = stmt {
                    retains_item(item)
                } else {
                    true
                }
            });

            visit_mut::visit_block_mut(self, node);
        }
    }

    fn extract_attrs(item: &Item) -> Option<&Vec<Attribute>> {
        Some(match item {
            Item::Const(i_const) => &i_const.attrs,
            Item::Enum(i_enum) => &i_enum.attrs,
            Item::ExternCrate(i_extern_crate) => &i_extern_crate.attrs,
            Item::Fn(i_fn) => &i_fn.attrs,
            Item::ForeignMod(i_foreign_mod) => &i_foreign_mod.attrs,
            Item::Impl(i_impl) => &i_impl.attrs,
            Item::Macro(i_macro) => &i_macro.attrs,
            Item::Macro2(i_macro2) => &i_macro2.attrs,
            Item::Mod(i_mod) => &i_mod.attrs,
            Item::Static(i_static) => &i_static.attrs,
            Item::Struct(i_struct) => &i_struct.attrs,
            Item::Trait(i_trait) => &i_trait.attrs,
            Item::TraitAlias(i_trait_alias) => &i_trait_alias.attrs,
            Item::Type(i_type) => &i_type.attrs,
            Item::Union(i_union) => &i_union.attrs,
            Item::Use(i_use) => &i_use.attrs,
            _ => return None,
        })
    }

    fn retains_item(item: &Item) -> bool {
        let attrs = match extract_attrs(item) {
            Some(attrs) => attrs,
            None => return true,
        };

        attrs.iter().all(|attr| {
            // parse #[test]
            if_chain! {
                if let Some(name) = attr.path.get_ident();
                if name == "test";
                then {
                    return false;
                }
            }

            // parse #[cfg(test)]
            if_chain! {
                if let Ok(meta) = attr.parse_meta();
                if let Meta::List(list) = meta;
                if let Some(name) = list.path.get_ident();
                if name == "cfg";
                then {
                    for nest in list.nested.iter() {
                        let path = match nest {
                            NestedMeta::Meta(Meta::Path(path)) => path,
                            _ => continue,
                        };

                        if_chain! {
                            if let Some(ident) = path.get_ident();
                            if ident == "test";
                            then {
                                return false;
                            }
                        }
                    }
                }
            }

            true
        })
    }
}

fn remove_cfg_version(ver: RustVersion, file: &mut syn::File) -> anyhow::Result<()> {
    let features = match ver {
        RustVersion::Rust2016 => &["rust-151"] as &[&str],
        RustVersion::Rust2020 => &[
            "rust-151",
            "rust-130",
            "rust-131",
            "rust-142",
            "crates-atc-2020",
        ],
    };

    let mut remover = ItemRemover::new(features);
    remover.visit_file_mut(file);

    return if remover.parse_errors.is_empty() {
        Ok(())
    } else {
        // FIXME: なんかもっといい感じの構造に保持すべき
        Err(anyhow!(
            "{} parse error(s): {:?}",
            remover.parse_errors.len(),
            remover.parse_errors
        ))
    };

    use syn::visit_mut;
    use syn::visit_mut::VisitMut;
    use syn::*;

    struct ItemRemover {
        features: &'static [&'static str],
        parse_errors: Vec<anyhow::Error>,
    };

    impl ItemRemover {
        fn new(features: &'static [&'static str]) -> ItemRemover {
            ItemRemover {
                features,
                parse_errors: Vec::new(),
            }
        }
    }

    impl VisitMut for ItemRemover {
        fn visit_file_mut(&mut self, node: &mut File) {
            node.items.retain(|item| {
                let retains = retains_item(item, self.features);
                match retains {
                    Err(e) => {
                        self.parse_errors.push(e);
                        false
                    }
                    Ok(r) => r,
                }
            });
            node.items.iter_mut().for_each(remove_cfg_feature_from_item);

            visit_mut::visit_file_mut(self, node);
        }

        fn visit_item_mod_mut(&mut self, node: &mut ItemMod) {
            if let Some((_, items)) = &mut node.content {
                items.retain(|item| {
                    let retains = retains_item(item, self.features);
                    match retains {
                        Err(e) => {
                            self.parse_errors.push(e);
                            false
                        }
                        Ok(r) => r,
                    }
                });
                items.iter_mut().for_each(remove_cfg_feature_from_item);
            }

            visit_mut::visit_item_mod_mut(self, node);
        }

        fn visit_block_mut(&mut self, node: &mut Block) {
            node.stmts.retain(|stmt| {
                let retains = retains_stmt(stmt, self.features);
                match retains {
                    Err(e) => {
                        self.parse_errors.push(e);
                        false
                    }
                    Ok(r) => r,
                }
            });
            node.stmts.iter_mut().for_each(remove_cfg_feature_from_stmt);

            visit_mut::visit_block_mut(self, node);
        }
    }

    fn extract_attrs_from_item(item: &Item) -> Option<&Vec<Attribute>> {
        Some(match item {
            Item::Const(i_const) => &i_const.attrs,
            Item::Enum(i_enum) => &i_enum.attrs,
            Item::ExternCrate(i_extern_crate) => &i_extern_crate.attrs,
            Item::Fn(i_fn) => &i_fn.attrs,
            Item::ForeignMod(i_foreign_mod) => &i_foreign_mod.attrs,
            Item::Impl(i_impl) => &i_impl.attrs,
            Item::Macro(i_macro) => &i_macro.attrs,
            Item::Macro2(i_macro2) => &i_macro2.attrs,
            Item::Mod(i_mod) => &i_mod.attrs,
            Item::Static(i_static) => &i_static.attrs,
            Item::Struct(i_struct) => &i_struct.attrs,
            Item::Trait(i_trait) => &i_trait.attrs,
            Item::TraitAlias(i_trait_alias) => &i_trait_alias.attrs,
            Item::Type(i_type) => &i_type.attrs,
            Item::Union(i_union) => &i_union.attrs,
            Item::Use(i_use) => &i_use.attrs,
            _ => return None,
        })
    }

    fn extract_attrs_from_item_mut(item: &mut Item) -> Option<&mut Vec<Attribute>> {
        Some(match item {
            Item::Const(i_const) => &mut i_const.attrs,
            Item::Enum(i_enum) => &mut i_enum.attrs,
            Item::ExternCrate(i_extern_crate) => &mut i_extern_crate.attrs,
            Item::Fn(i_fn) => &mut i_fn.attrs,
            Item::ForeignMod(i_foreign_mod) => &mut i_foreign_mod.attrs,
            Item::Impl(i_impl) => &mut i_impl.attrs,
            Item::Macro(i_macro) => &mut i_macro.attrs,
            Item::Macro2(i_macro2) => &mut i_macro2.attrs,
            Item::Mod(i_mod) => &mut i_mod.attrs,
            Item::Static(i_static) => &mut i_static.attrs,
            Item::Struct(i_struct) => &mut i_struct.attrs,
            Item::Trait(i_trait) => &mut i_trait.attrs,
            Item::TraitAlias(i_trait_alias) => &mut i_trait_alias.attrs,
            Item::Type(i_type) => &mut i_type.attrs,
            Item::Union(i_union) => &mut i_union.attrs,
            Item::Use(i_use) => &mut i_use.attrs,
            _ => return None,
        })
    }

    fn extract_attrs_from_stmt(stmt: &Stmt) -> Option<&Vec<Attribute>> {
        match stmt {
            Stmt::Local(s_local) => Some(&s_local.attrs),
            Stmt::Item(s_item) => extract_attrs_from_item(s_item),
            Stmt::Expr(s_expr) => extract_attrs_from_expr(&s_expr),
            Stmt::Semi(s_expr, _) => extract_attrs_from_expr(&s_expr),
        }
    }

    fn extract_attrs_from_stmt_mut(stmt: &mut Stmt) -> Option<&mut Vec<Attribute>> {
        match stmt {
            Stmt::Local(s_local) => Some(&mut s_local.attrs),
            Stmt::Item(s_item) => extract_attrs_from_item_mut(s_item),
            Stmt::Expr(s_expr) => extract_attrs_from_expr_mut(&mut *s_expr),
            Stmt::Semi(s_expr, _) => extract_attrs_from_expr_mut(&mut *s_expr),
        }
    }

    fn extract_attrs_from_expr(expr: &Expr) -> Option<&Vec<Attribute>> {
        match expr {
            Expr::Array(inner) => Some(&inner.attrs),
            Expr::Assign(inner) => Some(&inner.attrs),
            Expr::AssignOp(inner) => Some(&inner.attrs),
            Expr::Async(inner) => Some(&inner.attrs),
            Expr::Await(inner) => Some(&inner.attrs),
            Expr::Binary(inner) => Some(&inner.attrs),
            Expr::Block(inner) => Some(&inner.attrs),
            Expr::Box(inner) => Some(&inner.attrs),
            Expr::Break(inner) => Some(&inner.attrs),
            Expr::Call(inner) => Some(&inner.attrs),
            Expr::Cast(inner) => Some(&inner.attrs),
            Expr::Closure(inner) => Some(&inner.attrs),
            Expr::Continue(inner) => Some(&inner.attrs),
            Expr::Field(inner) => Some(&inner.attrs),
            Expr::ForLoop(inner) => Some(&inner.attrs),
            Expr::Group(inner) => Some(&inner.attrs),
            Expr::If(inner) => Some(&inner.attrs),
            Expr::Index(inner) => Some(&inner.attrs),
            Expr::Let(inner) => Some(&inner.attrs),
            Expr::Lit(inner) => Some(&inner.attrs),
            Expr::Loop(inner) => Some(&inner.attrs),
            Expr::Macro(inner) => Some(&inner.attrs),
            Expr::Match(inner) => Some(&inner.attrs),
            Expr::MethodCall(inner) => Some(&inner.attrs),
            Expr::Paren(inner) => Some(&inner.attrs),
            Expr::Path(inner) => Some(&inner.attrs),
            Expr::Range(inner) => Some(&inner.attrs),
            Expr::Reference(inner) => Some(&inner.attrs),
            Expr::Repeat(inner) => Some(&inner.attrs),
            Expr::Return(inner) => Some(&inner.attrs),
            Expr::Struct(inner) => Some(&inner.attrs),
            Expr::Try(inner) => Some(&inner.attrs),
            Expr::TryBlock(inner) => Some(&inner.attrs),
            Expr::Tuple(inner) => Some(&inner.attrs),
            Expr::Type(inner) => Some(&inner.attrs),
            Expr::Unary(inner) => Some(&inner.attrs),
            Expr::Unsafe(inner) => Some(&inner.attrs),
            Expr::Verbatim(_) => None,
            Expr::While(inner) => Some(&inner.attrs),
            Expr::Yield(inner) => Some(&inner.attrs),
            _ => unreachable!(),
        }
    }

    fn extract_attrs_from_expr_mut(expr: &mut Expr) -> Option<&mut Vec<Attribute>> {
        match expr {
            Expr::Array(inner) => Some(&mut inner.attrs),
            Expr::Assign(inner) => Some(&mut inner.attrs),
            Expr::AssignOp(inner) => Some(&mut inner.attrs),
            Expr::Async(inner) => Some(&mut inner.attrs),
            Expr::Await(inner) => Some(&mut inner.attrs),
            Expr::Binary(inner) => Some(&mut inner.attrs),
            Expr::Block(inner) => Some(&mut inner.attrs),
            Expr::Box(inner) => Some(&mut inner.attrs),
            Expr::Break(inner) => Some(&mut inner.attrs),
            Expr::Call(inner) => Some(&mut inner.attrs),
            Expr::Cast(inner) => Some(&mut inner.attrs),
            Expr::Closure(inner) => Some(&mut inner.attrs),
            Expr::Continue(inner) => Some(&mut inner.attrs),
            Expr::Field(inner) => Some(&mut inner.attrs),
            Expr::ForLoop(inner) => Some(&mut inner.attrs),
            Expr::Group(inner) => Some(&mut inner.attrs),
            Expr::If(inner) => Some(&mut inner.attrs),
            Expr::Index(inner) => Some(&mut inner.attrs),
            Expr::Let(inner) => Some(&mut inner.attrs),
            Expr::Lit(inner) => Some(&mut inner.attrs),
            Expr::Loop(inner) => Some(&mut inner.attrs),
            Expr::Macro(inner) => Some(&mut inner.attrs),
            Expr::Match(inner) => Some(&mut inner.attrs),
            Expr::MethodCall(inner) => Some(&mut inner.attrs),
            Expr::Paren(inner) => Some(&mut inner.attrs),
            Expr::Path(inner) => Some(&mut inner.attrs),
            Expr::Range(inner) => Some(&mut inner.attrs),
            Expr::Reference(inner) => Some(&mut inner.attrs),
            Expr::Repeat(inner) => Some(&mut inner.attrs),
            Expr::Return(inner) => Some(&mut inner.attrs),
            Expr::Struct(inner) => Some(&mut inner.attrs),
            Expr::Try(inner) => Some(&mut inner.attrs),
            Expr::TryBlock(inner) => Some(&mut inner.attrs),
            Expr::Tuple(inner) => Some(&mut inner.attrs),
            Expr::Type(inner) => Some(&mut inner.attrs),
            Expr::Unary(inner) => Some(&mut inner.attrs),
            Expr::Unsafe(inner) => Some(&mut inner.attrs),
            Expr::Verbatim(_) => None,
            Expr::While(inner) => Some(&mut inner.attrs),
            Expr::Yield(inner) => Some(&mut inner.attrs),
            _ => unreachable!(),
        }
    }

    fn retains_item(item: &Item, features: &[&str]) -> anyhow::Result<bool> {
        extract_attrs_from_item(item)
            .map(|attrs| retains(attrs, features))
            .unwrap_or(Ok(true))
    }

    fn retains_stmt(stmt: &Stmt, features: &[&str]) -> anyhow::Result<bool> {
        extract_attrs_from_stmt(stmt)
            .map(|attrs| retains(attrs, features))
            .unwrap_or(Ok(true))
    }

    fn retains(attrs: &[Attribute], features: &[&str]) -> anyhow::Result<bool> {
        let cfgs = attrs
            .iter()
            .filter_map(|attr| parse_cfg_attr(attr).transpose())
            .collect::<anyhow::Result<Vec<_>>>()?;
        let matches = cfgs.into_iter().all(|cfg| cfg.match_features(features));
        Ok(matches)
    }

    enum Cfg {
        NameValue { name: String, value: String },
        And(Vec<Cfg>),
        Or(Vec<Cfg>),
        Not(Box<Cfg>),
    }

    impl Cfg {
        fn match_features(&self, features: &[&str]) -> bool {
            match self {
                Cfg::NameValue { name, value } => name != "feature" || features.contains(&&**value),
                Cfg::And(xs) => xs.iter().all(|x| x.match_features(features)),
                Cfg::Or(xs) => xs.iter().any(|x| x.match_features(features)),
                Cfg::Not(x) => !x.match_features(features),
            }
        }
    }

    fn parse_cfg_inner(nested: &NestedMeta) -> anyhow::Result<Cfg> {
        let meta = match nested {
            NestedMeta::Meta(meta) => meta,
            _ => bail!("cfg() has no nested meta"),
        };
        match meta {
            Meta::NameValue(meta) => if_chain! {
                if let Some(ident) = meta.path.get_ident();
                let name = ident.to_string();
                if let Lit::Str(ref lit) = meta.lit;
                let value = lit.value();
                then {
                    return Ok(Cfg::NameValue { name, value });
                }
            },
            Meta::List(list) => if_chain! {
                if let Some(ident) = list.path.get_ident();
                let pred = ident.to_string();
                then {
                    let args = list
                        .nested
                        .iter()
                        .map(parse_cfg_inner)
                        .collect::<anyhow::Result<_>>()?;
                    return match &*pred {
                        "and" => Ok(Cfg::And(args)),
                        "or" => Ok(Cfg::Or(args)),
                        "not" => {
                            ensure!(
                                args.len() == 1,
                                "unexpected number of arguments for not() predicate",
                            );
                            Ok(Cfg::Not(Box::new(args.into_iter().next().unwrap())))
                        },
                        _ => bail!("unknown predicate"),
                    };
                }
            },
            _ => {}
        }

        Err(anyhow!("failed to parse cfg() arguments"))
    }

    fn parse_cfg_attr(attr: &Attribute) -> anyhow::Result<Option<Cfg>> {
        // parse #[cfg(feature = "...")]
        // parse #[cfg(not(feature = "..."))]
        if_chain! {
            if let Ok(meta) = attr.parse_meta();
            if let Meta::List(list) = meta;
            if let Some(name) = list.path.get_ident();
            if name == "cfg";
            then {
                ensure!(
                    list.nested.len() == 1,
                    "unexpected number of arguments for cfg(): found {}",
                    list.nested.len()
                );
                Some(parse_cfg_inner(&list.nested[0])).transpose()
            } else {
                Ok(None)
            }
        }
    }

    fn remove_cfg_feature_from_item(item: &mut Item) {
        if let Some(attrs) = extract_attrs_from_item_mut(item) {
            remove_cfg_feature(attrs)
        }
    }

    fn remove_cfg_feature_from_stmt(stmt: &mut Stmt) {
        if let Some(attrs) = extract_attrs_from_stmt_mut(stmt) {
            remove_cfg_feature(attrs)
        }
    }

    fn remove_cfg_feature(attrs: &mut Vec<Attribute>) {
        attrs.retain(|attr| matches!(parse_cfg_attr(attr), Ok(cfg) if cfg.is_none()));
    }
}

fn rustfmt(source: &str) -> Result<String> {
    let mut child = Command::new("rustup")
        .arg("run")
        .arg("stable")
        .arg("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("failed to get stdin"))?;
    stdin.write_all(source.as_bytes())?;
    drop(stdin);
    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[allow(clippy::unnecessary_wraps)]
fn minify(source: &str) -> Result<String> {
    let replaces = [
        (&*RE_MULTIPLE_SPACE, " "),
        (&*RE_WHITESPACE_AFTER_COLONS, "$col"),
        (&*RE_WHITESPACE_AROUND_OPERATOR, "$op"),
        (&*RE_WHITESPACE_AROUND_PAREN, "$par"),
    ];

    let mut result = source
        .lines()
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .join(" ");
    for &(regex, replace) in replaces.iter() {
        let replaced = regex.replace_all(&result, replace);
        let replaced = replaced.trim();
        result = replaced.to_string();
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_cfg_version() {
        let mut cond_true: syn::File = parse_quote! {
            #[cfg(feature = "rust-142")]
            fn foo() {}
        };
        remove_cfg_version(RustVersion::Rust2020, &mut cond_true).unwrap();

        let mut not_false: syn::File = parse_quote! {
            #[cfg(not(feature = "rust-142"))]
            fn foo() {}
        };
        remove_cfg_version(RustVersion::Rust2020, &mut not_false).unwrap();

        let mut and_false: syn::File = parse_quote! {
            #[cfg(and(feature = "rust-142", not(feature = "rust-142")))]
            fn foo() {}
        };
        remove_cfg_version(RustVersion::Rust2020, &mut and_false).unwrap();

        let mut and_true: syn::File = parse_quote! {
            #[cfg(and(feature = "crates-atc-2020", feature = "rust-142"))]
            fn foo() {}
        };
        remove_cfg_version(RustVersion::Rust2020, &mut and_true).unwrap();

        let mut or_false: syn::File = parse_quote! {
            #[cfg(or(not(feature = "crates-atc-2020"), not(feature = "rust-142")))]
            fn foo() {}
        };
        remove_cfg_version(RustVersion::Rust2020, &mut or_false).unwrap();

        let mut or_true: syn::File = parse_quote! {
            #[cfg(or(feature = "crates-atc-2020", not(feature = "rust-142")))]
            fn foo() {}
        };
        remove_cfg_version(RustVersion::Rust2020, &mut or_true).unwrap();

        assert!(!cond_true.into_token_stream().is_empty());
        assert!(not_false.into_token_stream().is_empty());
        assert!(and_false.into_token_stream().is_empty());
        assert!(!and_true.into_token_stream().is_empty());
        assert!(or_false.into_token_stream().is_empty());
        assert!(!or_true.into_token_stream().is_empty());
    }
}

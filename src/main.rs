mod api_client;

use crate::api_client::Realisation;
use askama::Template;
use reqwest::Url;
use std::path::Path;
use std::process::{self, Command};
use std::{env, fs, io};

const LOCAL_BASE_URL: &'static str = "http://localhost:8055";
const LOCAL_API_KEY: &'static str = "iMrfmSbhlhA-fagQ5DB7T0_8TbqkWmBY";

#[derive(Template)]
#[template(path = "index.jinja2", ext = "html")]
struct TemplateIndex<'a> {
    title: String,
    nav_links: &'a Vec<&'a NavLink<'a>>,
    current_link: &'a NavLink<'a>,
    realisations: &'a Vec<Realisation>,
}

#[derive(Template)]
#[template(path = "realisaties.jinja2", ext = "html")]
struct TemplateRealisations<'a> {
    title: String,
    nav_links: &'a Vec<&'a NavLink<'a>>,
    current_link: &'a NavLink<'a>,
    realisation: &'a Realisation,
}

struct NavLink<'a> {
    name: &'static str,
    url: &'static str,
    children: Option<Vec<&'a NavLink<'a>>>,
}

fn main() {
    // Setup logger
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // Collect env vars
    let base_url = env_var_with_default("WTECH_BASE_URL", LOCAL_BASE_URL);
    let api_key = env_var_with_default("WTECH_API_KEY", LOCAL_API_KEY);

    // Create HTTP client
    let client = api_client::get_api_client(&api_key);
    let base_url = Url::parse(&base_url).unwrap();

    // Fetch realisations
    let realisations = api_client::get_realisations(&client, &base_url);

    // Prepare output dir
    let path_output = Path::new("output");
    let path_assets = Path::new("assets");
    ensure_empty_dir(path_output).expect("Unable to ensure empty output directory");
    copy_assets(&path_assets.join("."), path_output).expect("Unable to copy assets");

    // Nav links
    let nav_link_start = NavLink {
        name: "Start",
        url: "/",
        children: None,
    };
    let nav_link_realisaties_aircoheaters = NavLink {
        name: "Aircoheaters",
        url: "/realisaties/aircoheaters",
        children: None,
    };
    let nav_link_realisaties_warmtepompen = NavLink {
        name: "Warmtepompen",
        url: "/realisaties/warmtepompen",
        children: None,
    };
    let nav_link_realisaties_ventilatie = NavLink {
        name: "Ventilatie",
        url: "/realisaties/ventilatie",
        children: None,
    };
    let nav_link_realisaties = NavLink {
        name: "Realisaties",
        url: "/realisaties",
        children: Some(vec![
            &nav_link_realisaties_aircoheaters,
            &nav_link_realisaties_warmtepompen,
            &nav_link_realisaties_ventilatie,
        ]),
    };
    let nav_links = vec![&nav_link_start, &nav_link_realisaties];

    // Generate templates
    fs::write(
        path_output.join("index.html"),
        TemplateIndex {
            title: "Start".to_string(),
            nav_links: &nav_links,
            current_link: &nav_link_start,
            realisations: &realisations,
        }
        .render()
        .expect("Unable to render index template"),
    )
    .expect("Failed to write index.html");

    let path_realisaties = path_output.join("realisaties");
    for realisation in &realisations {
        // download_image(&config, &realisation.main_image).await;
        let path_realisation = path_realisaties.join(&realisation.slug);
        fs::create_dir_all(path_realisation.clone()).expect("Failed to create realisation dir");
        fs::write(
            path_realisation.join("index.html"),
            TemplateRealisations {
                title: realisation.name.clone(),
                nav_links: &nav_links,
                current_link: &nav_link_start,
                realisation: &realisation,
            }
            .render()
            .expect("Unable to render index template"),
        )
        .expect("Failed to write index.html");
    }
}

fn env_var_with_default(name: &'static str, default: &'static str) -> String {
    env::var(name).unwrap_or_else(|_| {
        log::info!("Unable to read {name}. Using default: {default}");
        default.to_string()
    })
}

fn ensure_empty_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn copy_assets(source: &Path, target: &Path) -> io::Result<process::Output> {
    Command::new("cp")
        .args([
            "--recursive",
            "--dereference",
            "--preserve=all",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .output()
}

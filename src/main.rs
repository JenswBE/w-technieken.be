mod api_client;

use crate::api_client::{Client, Realisation};
use askama::Template;
use dotenv::dotenv;
use rayon::prelude::*;
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
    nav_links: &'a Vec<&'a NavLink>,
    current_link: &'a NavLink,
    realisations: &'a Vec<Realisation>,
}

#[derive(Template)]
#[template(path = "over_ons.jinja2", ext = "html")]
struct TemplateAboutUs<'a> {
    title: String,
    nav_links: &'a Vec<&'a NavLink>,
    current_link: &'a NavLink,
}

#[derive(Template)]
#[template(path = "realisaties.jinja2", ext = "html")]
struct TemplateRealisations<'a> {
    title: String,
    nav_links: &'a Vec<&'a NavLink>,
    current_link: &'a NavLink,
    realisation: &'a Realisation,
}

struct NavLink {
    name: String,
    url: String,
    children: Option<Vec<NavLink>>,
}

fn main() {
    // Load .env file
    dotenv().ok();

    // Setup logger
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // Collect env vars
    let base_url = env_var_with_default("WTECH_BASE_URL", LOCAL_BASE_URL);
    let api_key = env_var_with_default("WTECH_API_KEY", LOCAL_API_KEY);

    // Create HTTP client
    let base_url = Url::parse(&base_url).unwrap();
    let client = Client::build(base_url, &api_key);

    // Fetch realisations
    let realisations = client.get_realisations();

    // Prepare output dir
    let path_output = Path::new("output");
    let path_static = Path::new("static");
    ensure_empty_dir(path_output).expect("Unable to ensure empty output directory");
    copy_static(&path_static.join("."), path_output).expect("Unable to copy statics");

    // Nav links
    let nav_link_start = NavLink {
        name: "Start".to_string(),
        url: "/".to_string(),
        children: None,
    };
    let nav_link_about_us = NavLink {
        name: "Over ons".to_string(),
        url: "/over-ons".to_string(),
        children: None,
    };
    let nav_link_realisaties = NavLink {
        name: "Realisaties".to_string(),
        url: "/realisaties".to_string(),
        children: Some(
            realisations
                .iter()
                .map(|r| NavLink {
                    name: r.name.clone(),
                    url: r.slug.clone(),
                    children: None,
                })
                .collect(),
        ),
    };
    let nav_links = vec![&nav_link_start, &nav_link_about_us, &nav_link_realisaties];

    // Generate index page
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

    // Generate "About us" page
    let path_over_ons = path_output.join("over-ons");
    fs::create_dir_all(&path_over_ons).expect("Failed to create over-ons dir");
    fs::write(
        path_output.join("index.html"),
        TemplateAboutUs {
            title: "Over ons".to_string(),
            nav_links: &nav_links,
            current_link: &nav_link_about_us,
        }
        .render()
        .expect("Unable to render index template"),
    )
    .expect("Failed to write index.html");

    // Generate realisation pages
    let mut asset_download_queue = vec![];
    let path_assets = path_output.join("assets");
    fs::create_dir_all(&path_assets).expect("Failed to create output assets dir");
    let path_realisaties = path_output.join("realisaties");
    for realisation in &realisations {
        // Queue asset download - Index realisatie
        asset_download_queue.push(DownloadAsset {
            id: realisation.main_image.clone(),
            extension: "jpg",
            key: Some("index-realisatie"),
        });

        // Queue asset download - Realisatie
        if let Some(secondary_images) = &realisation.secondary_images {
            for image_id in secondary_images {
                asset_download_queue.push(DownloadAsset {
                    id: image_id.clone(),
                    extension: "jpg",
                    key: Some("realisatie-full"),
                });
                asset_download_queue.push(DownloadAsset {
                    id: image_id.clone(),
                    extension: "jpg",
                    key: Some("realisatie-thumbnail"),
                });
            }
        }

        // Generate page
        let path_realisation = path_realisaties.join(&realisation.slug);
        fs::create_dir_all(&path_realisation).expect("Failed to create realisation dir");
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

    // Download assets
    asset_download_queue.par_iter().for_each(|asset| {
        client.download_asset(&path_assets, &asset.id, &asset.extension, asset.key, None)
    })
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

fn copy_static(source: &Path, target: &Path) -> io::Result<process::Output> {
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

struct DownloadAsset {
    id: String,
    extension: &'static str,
    key: Option<&'static str>,
}

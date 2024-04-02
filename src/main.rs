mod api_client;

use crate::api_client::{Client, GeneralSettings, Realisation};
use askama::Template;
use dotenv::dotenv;
use rayon::prelude::*;
use reqwest::Url;
use std::path::Path;
use std::process::{self, Command};
use std::{env, fs, io};

const LOCAL_BASE_URL: &'static str = "http://localhost:8055";
const LOCAL_API_KEY: &'static str = "iMrfmSbhlhA-fagQ5DB7T0_8TbqkWmBY";

/// Fields present in each template with the same value.
struct TemplateBaseCommon<'a> {
    nav_links: &'a Vec<&'a NavLink>,
    email: &'a str,
    phone_number: &'a str,
    vat_number: &'a str,
}

/// Fields present in each template but with a different value.
struct TemplateBaseSpecific<'a> {
    title: String,
    current_link: &'a NavLink,
}

#[derive(Template)]
#[template(path = "index.jinja2", ext = "html")]
struct TemplateIndex<'a> {
    base_common: &'a TemplateBaseCommon<'a>,
    base_specific: TemplateBaseSpecific<'a>,
    main_image_id: String,
    realisations: &'a Vec<Realisation>,
}

#[derive(Template)]
#[template(path = "realisaties.jinja2", ext = "html")]
struct TemplateRealisations<'a> {
    base_common: &'a TemplateBaseCommon<'a>,
    base_specific: TemplateBaseSpecific<'a>,
    realisation: &'a Realisation,
}

#[derive(Template)]
#[template(path = "over_ons.jinja2", ext = "html")]
struct TemplateAboutUs<'a> {
    base_common: &'a TemplateBaseCommon<'a>,
    base_specific: TemplateBaseSpecific<'a>,
}

#[derive(Template)]
#[template(path = "onze_diensten.jinja2", ext = "html")]
struct TemplateOurServices<'a> {
    base_common: &'a TemplateBaseCommon<'a>,
    base_specific: TemplateBaseSpecific<'a>,
}

#[derive(Template)]
#[template(path = "404.jinja2", ext = "html")]
struct Template404<'a> {
    base_common: &'a TemplateBaseCommon<'a>,
    base_specific: TemplateBaseSpecific<'a>,
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
    let path_cache = env::var("WTECH_CACHE_DIR").ok();

    // Create HTTP client
    let base_url = Url::parse(&base_url).unwrap();
    let client = Client::build(base_url, &api_key);

    // Fetch remote data
    let general_settings = client.get_general_settings();
    let realisations = client.get_realisations();

    // Prepare asset cache dir
    let path_cache = path_cache.as_ref().map(Path::new);
    let path_cache_assets = path_cache.map(|p| p.join("assets"));
    if let Some(path_cache_assets) = &path_cache_assets {
        log::info!(
            "Caching enabled to folder \"{}\"",
            path_cache.unwrap().display()
        );
        fs::create_dir_all(&path_cache_assets).expect("Failed to create assets cache dir");
    }

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
    let nav_link_realisaties = NavLink {
        name: "Realisaties".to_string(),
        url: "/realisaties".to_string(),
        children: Some(
            realisations
                .iter()
                .map(|r| NavLink {
                    name: r.name.clone(),
                    url: "/realisaties/".to_string() + &r.slug,
                    children: None,
                })
                .collect(),
        ),
    };
    let nav_link_about_us = NavLink {
        name: "Over ons".to_string(),
        url: "/over-ons".to_string(),
        children: None,
    };
    let nav_link_our_services = NavLink {
        name: "Onze diensten".to_string(),
        url: "/onze-diensten".to_string(),
        children: None,
    };
    let nav_links = vec![
        &nav_link_start,
        &nav_link_realisaties,
        &nav_link_about_us,
        &nav_link_our_services,
    ];

    // Base template
    let base_template_common = TemplateBaseCommon {
        nav_links: &nav_links,
        email: &general_settings.email,
        phone_number: &general_settings.phone_number,
        vat_number: &general_settings.vat_number,
    };

    // Ensure assets dir exists
    let path_assets = path_output.join("assets");
    fs::create_dir_all(&path_assets).expect("Failed to create output assets dir");

    // Generate index page
    let index_main_image_id = general_settings
        .start_image
        .expect("Start image must be defined in general settings")
        .id
        .into_inner();
    client.download_asset(
        &path_assets,
        &index_main_image_id,
        "jpg",
        Some("index-main-image"),
        path_cache_assets.as_ref(),
    );
    fs::write(
        path_output.join("index.html"),
        TemplateIndex {
            base_common: &base_template_common,
            base_specific: TemplateBaseSpecific {
                title: "Start".to_string(),
                current_link: &nav_link_start,
            },
            main_image_id: index_main_image_id,
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
        path_over_ons.join("index.html"),
        TemplateAboutUs {
            base_common: &base_template_common,
            base_specific: TemplateBaseSpecific {
                title: "Over ons".to_string(),
                current_link: &nav_link_about_us,
            },
        }
        .render()
        .expect("Unable to render index template"),
    )
    .expect("Failed to write index.html");

    // Generate "About us" page
    let path_over_ons = path_output.join("onze-diensten");
    fs::create_dir_all(&path_over_ons).expect("Failed to create onze-diensten dir");
    fs::write(
        path_over_ons.join("index.html"),
        TemplateOurServices {
            base_common: &base_template_common,
            base_specific: TemplateBaseSpecific {
                title: "Onze diensten".to_string(),
                current_link: &nav_link_our_services,
            },
        }
        .render()
        .expect("Unable to render index template"),
    )
    .expect("Failed to write index.html");

    // Generate realisation pages
    let mut asset_download_queue = vec![];
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
                base_common: &base_template_common,
                base_specific: TemplateBaseSpecific {
                    title: realisation.name.clone(),
                    current_link: &nav_link_realisaties,
                },
                realisation: &realisation,
            }
            .render()
            .expect("Unable to render index template"),
        )
        .expect("Failed to write index.html");
    }

    // Download assets
    asset_download_queue.par_iter().for_each(|asset| {
        client.download_asset(
            &path_assets,
            &asset.id,
            &asset.extension,
            asset.key,
            path_cache_assets.as_ref(),
        )
    });

    // Generate "404" page
    fs::write(
        path_output.join("404.html"),
        Template404 {
            base_common: &base_template_common,
            base_specific: TemplateBaseSpecific {
                title: "Pagina niet gevonden".to_string(),
                current_link: &nav_link_start,
            },
        }
        .render()
        .expect("Unable to render 404 template"),
    )
    .expect("Failed to write 404.html");
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

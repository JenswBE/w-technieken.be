use cynic::QueryBuilder;
use reqwest::{blocking::Client, header, Url};
use std::path::Path;
use std::process::{self, Command};
use std::{fs, io};

use askama::Template;

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

struct Realisation {
    name: String,
    slug: String,
    slogan: Option<String>,
    main_image: String,
    // secondary_images: Vec<String>,
}

impl From<Realisations> for Realisation {
    fn from(item: Realisations) -> Self {
        Self {
            name: item.name.expect("Realisation must have a name"),
            slug: item.slug.expect("Realisation must have a slug"),
            slogan: item.slogan,
            main_image: item
                .main_image
                .expect("Realisation must have a main image")
                .id
                .into_inner(),
            // secondary_images: (),
        }
    }
}

fn main() {
    // Create HTTP client
    let client = get_api_client("iMrfmSbhlhA-fagQ5DB7T0_8TbqkWmBY");
    let base_url = Url::parse("http://localhost:8055").unwrap();

    // Fetch realisations
    let realisations = get_realisations(&client, &base_url)
        .data
        .expect("No realisations returned")
        .realisations;

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
    let realisations = realisations.into_iter().map(Realisation::from).collect();
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

// async fn download_image(config: &Configuration, id: &str) {
//     let file = openapi::apis::assets_api::get_asset(config, id, None, None, Some(true))
//         .await
//         .expect("Failed to get asset");
//     fs::write("output/test.jpg", file).expect("Failed to write image");
// }

#[cynic::schema("directus")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct AllRealisations {
    pub realisations: Vec<Realisations>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "realisations")]
pub struct Realisations {
    pub id: cynic::Id,
    pub name: Option<String>,
    pub slogan: Option<String>,
    pub slug: Option<String>,
    #[cynic(rename = "main_image")]
    pub main_image: Option<DirectusFiles>,
    #[cynic(rename = "additional_images")]
    pub additional_images: Option<Vec<Option<RealisationsFiles>>>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "realisations_files")]
pub struct RealisationsFiles {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "directus_files")]
pub struct DirectusFiles {
    pub id: cynic::Id,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_films_query_gql_output() {
        use cynic::QueryBuilder;

        let operation = AllRealisations::build(());

        insta::assert_snapshot!(operation.query);
    }
}

fn get_realisations(client: &Client, base_url: &Url) -> cynic::GraphQlResponse<AllRealisations> {
    use cynic::http::ReqwestBlockingExt;
    let graphql_url = base_url.join("/graphql").unwrap();
    client
        .post(graphql_url)
        .run_graphql(AllRealisations::build(()))
        .expect("Failed to fetch realisations")
}

fn get_api_client(api_token: &str) -> Client {
    let mut headers = header::HeaderMap::new();
    let mut auth_value = header::HeaderValue::from_str(&("Bearer ".to_string() + api_token))
        .expect("Unable to set authentication header");
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);
    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to create reqwest client")
}

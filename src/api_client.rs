use cynic::QueryBuilder;
use reqwest::{blocking, header, Url};
use std::fs::File;
use std::io::copy;
use std::path::Path;

pub struct Client {
    http_client: blocking::Client,
    base_url: Url,
}

impl Client {
    pub fn build(base_url: Url, api_token: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        let mut auth_value = header::HeaderValue::from_str(&("Bearer ".to_string() + api_token))
            .expect("Unable to set authentication header");
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);
        let http_client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .connection_verbose(true)
            .build()
            .expect("Failed to create reqwest client");

        Client {
            http_client,
            base_url,
        }
    }

    pub fn get_realisations(&self) -> Vec<Realisation> {
        use cynic::http::ReqwestBlockingExt;
        let graphql_url = self.base_url.join("/graphql").unwrap();
        let resp = self
            .http_client
            .post(graphql_url)
            .run_graphql(AllRealisations::build(()))
            .expect("Failed to fetch realisations");
        if let Some(errors) = resp.errors {
            for e in errors {
                log::error!("GraphQL query AllRealisations returned error(s): {e}")
            }
        };
        resp.data
            .expect("No realisations returned")
            .realisations
            .into_iter()
            .map(Realisation::from)
            .collect()
    }

    pub fn download_asset<P: AsRef<Path>>(
        &self,
        output_dir: P,
        id: &str,
        extension: &str,
        key: Option<&str>,
    ) {
        let filename = format!(
            "{}{}.{}",
            id,
            key.map_or("".to_string(), |k| "-".to_string() + k),
            extension
        );
        let asset_url = self
            .base_url
            .join(&format!("/assets/{}/{}", id, filename))
            .unwrap();
        log::info!("Downloading asset {} ...", filename);
        let mut req = self.http_client.get(asset_url.as_ref());
        if let Some(key) = key {
            req = req.query(&[("key", key), ("download", "true")]);
        }
        let mut resp = req.send().expect("Unable to get asset");
        if !resp.status().is_success() {
            log::error!(
                "Failed to fetch asset from {}: {}",
                asset_url,
                resp.text().unwrap()
            );
            panic!("Failed to fetch asset")
        }
        let output_path = output_dir.as_ref().join(filename);
        let mut file = File::create(&output_path)
            .expect(&format!("Unable to create file: {}", output_path.display()));
        copy(&mut resp, &mut file).expect("Unable to write asset to file");
    }
}

// Generated with https://generator.cynic-rs.dev/
#[cynic::schema("directus")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
struct AllRealisations {
    realisations: Vec<Realisations>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "realisations")]
struct Realisations {
    name: String,
    slogan: Option<String>,
    slug: String,
    #[cynic(rename = "main_image")]
    pub main_image: Option<DirectusFiles>,
    #[cynic(rename = "additional_images")]
    pub additional_images: Option<Vec<Option<RealisationsFiles>>>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "realisations_files")]
pub struct RealisationsFiles {
    #[cynic(rename = "directus_files_id")]
    pub directus_files_id: Option<DirectusFiles>,
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
    fn all_realisations_query_graphql_output() {
        use cynic::QueryBuilder;
        let operation = AllRealisations::build(());
        insta::assert_snapshot!(operation.query);
    }
}

pub struct Realisation {
    pub name: String,
    pub slug: String,
    pub slogan: Option<String>,
    pub main_image: String,
    pub secondary_images: Option<Vec<String>>,
}

impl From<Realisations> for Realisation {
    fn from(item: Realisations) -> Self {
        Self {
            name: item.name,
            slug: item.slug,
            slogan: item.slogan,
            main_image: item
                .main_image
                .expect("Realisation must have a main image")
                .id
                .into_inner(),
            secondary_images: item.additional_images.map(|images| {
                images
                    .into_iter()
                    .map(|file| {
                        file.expect("Additional image file cannot be None")
                            .directus_files_id
                            .expect("Additional image file must have Directus file ID")
                            .id
                            .into_inner()
                    })
                    .collect()
            }),
        }
    }
}

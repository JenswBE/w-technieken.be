use cynic::QueryBuilder;
use reqwest::{blocking::Client, header, Url};

pub fn get_api_client(api_token: &str) -> Client {
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

// fn download_asset(config: &Configuration, id: &str) {
//     let file = openapi::apis::assets_api::get_asset(config, id, None, None, Some(true))
//         .await
//         .expect("Failed to get asset");
//     fs::write("output/test.jpg", file).expect("Failed to write image");
// }

#[cynic::schema("directus")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
struct AllRealisations {
    pub realisations: Vec<Realisations>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "realisations")]
struct Realisations {
    pub name: Option<String>,
    pub slogan: Option<String>,
    pub slug: Option<String>,
    #[cynic(rename = "main_image")]
    pub main_image: Option<DirectusFiles>,
    #[cynic(rename = "additional_images")]
    pub additional_images: Option<Vec<Option<RealisationsFiles>>>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "directus_files")]
struct DirectusFiles {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "realisations_files")]
struct RealisationsFiles {
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

pub fn get_realisations(client: &Client, base_url: &Url) -> Vec<Realisation> {
    use cynic::http::ReqwestBlockingExt;
    let graphql_url = base_url.join("/graphql").unwrap();
    client
        .post(graphql_url)
        .run_graphql(AllRealisations::build(()))
        .expect("Failed to fetch realisations")
        .data
        .expect("No realisations returned")
        .realisations
        .into_iter()
        .map(Realisation::from)
        .collect()
}

pub struct Realisation {
    pub name: String,
    pub slug: String,
    pub slogan: Option<String>,
    pub main_image: String,
    // pub secondary_images: Vec<String>,
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

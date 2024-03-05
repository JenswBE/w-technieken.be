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
}

struct NavLink<'a> {
    name: &'static str,
    url: &'static str,
    children: Option<Vec<&'a NavLink<'a>>>,
}

fn main() {
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
        }
        .render()
        .expect("Unable to render index template"),
    )
    .expect("Failed to write index.html")
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

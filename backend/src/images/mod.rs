use std::fs as fss;
use std::path;

use actix_files as fs;
use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use rand::seq::IndexedRandom;

#[derive(Debug, MultipartForm)]
pub struct ImageUpload {
    #[multipart(rename = "post_image")]
    pub image: Option<TempFile>,
}

#[actix_web::get("/favicon")]
#[tracing::instrument(name = "Serving favicon", level = "info", target = "Static Content")]
pub async fn favicon() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving favicon");
    let filename = "head_shot.ico";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match fs::NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[actix_web::get("/icon-192")]
#[tracing::instrument(
    name = "Serving the icons-192",
    level = "info",
    target = "Static Content"
)]
pub async fn icon_192() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving icon-192");
    let filename = "icon-192.png";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match fs::NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[actix_web::get("/icon-512")]
#[tracing::instrument(
    name = "Serving the icons-512",
    level = "info",
    target = "Static Content"
)]
pub async fn icon_512() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving icon-512");
    let filename = "icon-512.png";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match fs::NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[actix_web::get("/icon_large")]
#[tracing::instrument(
    name = "Serving the icon_large",
    level = "info",
    target = "Static Content"
)]
pub async fn icon_large() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving icon_large");
    let filename = "icon_large.png";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match fs::NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[actix_web::get("/link_preview")]
#[tracing::instrument(
    name = "Serving the icon_large",
    level = "info",
    target = "Static Content"
)]
pub async fn link_preview() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving the link preview");
    let filename = "icon_1200x627.png";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match fs::NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[actix_web::get("/manifest.webmanifest")]
#[tracing::instrument(
    name = "Serving the manifest",
    level = "info",
    target = "Static Content"
)]
pub async fn manifest() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving manifest");
    let filename = "manifest.json";
    let path: path::PathBuf = ["static", filename].iter().collect();

    let file = match fs::NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[actix_web::get("/logomain")]
#[tracing::instrument(name = "Serving logo", level = "info", target = "Static Content")]
pub async fn logomain() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving logo");
    let filename = "logomain.jpeg";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match fs::NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[actix_web::get("/stylesheet")]
#[tracing::instrument(name = "Serving stylesheet", level = "info", target = "Static Content")]
pub async fn stylesheet() -> impl Responder {
    tracing::info!("Serving stylesheet");
    let file = include_str!("../../static/css/style.css");
    HttpResponse::Ok().content_type("text/css").body(file)
}

#[actix_web::get("/style.css.map")]
#[tracing::instrument(name = "Serving source map", level = "info", target = "Static Content")]
pub async fn source_map() -> impl Responder {
    tracing::info!("Serving source map");
    let file = include_str!("../../static/css/style.css.map");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(file)
}

#[actix_web::get("/htmx")]
#[tracing::instrument(
    name = "Serving htmx.min.js",
    level = "info",
    target = "Static Content"
)]
pub async fn htmx() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving htmx.min.js");

    let filename = "htmx.min.js";
    let path: path::PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match fs::NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[actix_web::get("/response-targets")]
#[tracing::instrument(
    name = "Serving response-targets.js",
    level = "info",
    target = "Static Content"
)]
pub async fn response_targets() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving response-targets.js");

    let filename = "response-targets.js";
    let pash: path::PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match fs::NamedFile::open(pash) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[actix_web::get("/sse")]
#[tracing::instrument(name = "Serving sse.js", level = "info", target = "Static Content")]
pub async fn sse() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving sse.js");

    let filename = "sse.js";
    let path: path::PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match fs::NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[actix_web::get("/action_script")]
#[tracing::instrument(
    name = "Serving action_script.js",
    level = "info",
    target = "Static Content"
)]
pub async fn action_script() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving action_script.js");

    let filename = "action_script.js";
    let path: path::PathBuf = ["static", "js", filename].iter().collect();

    match fs::NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[actix_web::get("/prof_headshot")]
#[tracing::instrument(
    name = "Serving prof_headshot.jpg",
    level = "info",
    target = "Static Content"
)]
pub async fn prof_headshot() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving prof_headshot.jpg");

    let filename = "head_shot.png";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    match fs::NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[actix_web::get("/usmc_patrolling")]
#[tracing::instrument(
    name = "Serving usmc_patrolling.jpg",
    level = "info",
    target = "Static Content"
)]
pub async fn usmc_patrolling() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving usmc_patrolling.jpg");

    let filename = "usmc_patrolling.jpg";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    match fs::NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[actix_web::get("/spinner")]
#[tracing::instrument(
    name = "Serving spinner.jpg",
    level = "info",
    target = "Static Content"
)]
pub async fn spinner() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving spinner.jpg");

    let filename = "spinner.gif";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    match fs::NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[actix_web::get("/github")]
#[tracing::instrument(name = "Serving github.svg", level = "info", target = "Static Content")]
pub async fn github() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving github.webp");

    let filename = "github.webp";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    match fs::NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[actix_web::get("/linkedin")]
#[tracing::instrument(
    name = "Serving linkedin.svg",
    level = "info",
    target = "Static Content"
)]
pub async fn linkedin() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving linkedin.svg");

    let filename = "linkedIn.svg";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    match fs::NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[actix_web::get("/settings_icon")]
#[tracing::instrument(
    name = "Serving settings_icon.jpg",
    level = "info",
    target = "settings_icon"
)]
pub async fn settings_icon() -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving settings_icon");

    let filename = "gears_001.jpg";
    let path: path::PathBuf = ["static", "imgs", filename].iter().collect();

    match fs::NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[actix_web::get("/rand_images/{num}")]
#[tracing::instrument(
    name = "Serving random_images to replace pravator dep.",
    level = "info",
    target = "Random images"
)]
pub async fn random_images(num: web::Path<u8>) -> Result<fs::NamedFile, actix_web::Error> {
    tracing::info!("Serving random images that replace pravatar");

    let mut rand_img: Vec<path::PathBuf> = Vec::new();

    let path: String = ["static/", "imgs/", "rand_set/"].concat();

    let dir = fss::read_dir(&path);
    if let Ok(entries) = dir {
        tracing::debug!("The found files: {entries:#?}");
        for entry in entries.filter_map(std::result::Result::ok) {
            let p = entry.path();
            if p.is_file() {
                rand_img.push(p);
            }
        }
    }

    if rand_img.is_empty() {
        tracing::error!("No images found in the requisite directory: {path}");
        return Err(actix_web::error::ErrorInternalServerError(
            "No images found in the requisite directory",
        ));
    }

    let mut rng = rand::rng();

    #[allow(clippy::unwrap_used)] // Failure here would be on the OS
    let file_chosen: path::PathBuf = rand_img.choose(&mut rng).cloned().unwrap();

    let path: path::PathBuf = std::iter::once(&&file_chosen).collect();
    tracing::warn!("The file chosen: {path:#?}");

    match fs::NamedFile::open(&path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening directory -- {path:#?} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

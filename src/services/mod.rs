use std::{
    io,
    path::{Path, PathBuf},
    sync::mpsc,
};

mod exif_data;
mod pair;
mod utils;

pub fn extract_and_apply_metadata(
    zip_path: &Path,
    rx: &mpsc::Receiver<()>,
    tx: &mpsc::Sender<Option<(PathBuf, io::Error)>>,
) {
    let working_dir = utils::unzip(zip_path);
    let file_names = utils::recursively_collect_filenames(&working_dir).unwrap();
    let pairs = pair::create_pairs(file_names);
    for pair in pairs.values() {
        let json = if let Some(json) = pair.read_json() {
            json.unwrap()
        } else {
            continue;
        };
        let exif = exif_data::TakeoutExif::from_json(json.as_str()).unwrap();
        for img in [&pair.img, &pair.img_edited]
            .into_iter()
            .filter_map(|i| i.clone())
        {
            if let Err(err) = exif.apply_to_image(&img) {
                tx.send(Some((img, err)))
                    .expect("Failed to send error to main thread");
                if let Err(err) = rx.recv() {
                    panic!("Failed to receive confirmation message: {}", err);
                }
            }
        }
    }
}

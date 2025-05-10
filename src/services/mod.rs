use std::path::Path;

mod exif_data;
mod pair;
mod utils;

pub fn extract_and_apply_metadata(zip_path: &Path) {
    let working_dir = utils::unzip(zip_path);
    let file_names = utils::recursively_collect_filenames(&working_dir).unwrap();
    let pairs = pair::create_pairs(file_names);
    for pair in pairs.values() {
        let json = pair.read_json().and_then(|inner| inner.ok()).unwrap();
        let exif = exif_data::TakeoutExif::from_json(json.as_str()).unwrap();
        for img in [&pair.img, &pair.img_edited].into_iter().filter_map(|i| i.clone()) {
            exif.apply_to_image(&img);
        }
    }
}

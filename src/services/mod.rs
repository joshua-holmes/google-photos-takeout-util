use std::path::Path;

mod exif_data;
mod pair;
mod utils;

pub fn extract_and_apply_metadata(zip_path: &Path) {
    let working_dir = utils::unzip(zip_path);
    let file_names = utils::recursively_collect_filenames(&working_dir).unwrap();
    let pairs = pair::create_pairs(file_names);
}

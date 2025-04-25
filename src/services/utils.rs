use std::{fs, io, path::Path};

pub fn unzip(zip_path: &Path) -> Vec<std::path::PathBuf> {
    let file = fs::File::open(zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let working_dir = zip_path.parent().unwrap().join(zip_path.file_stem().unwrap());
    fs::create_dir_all(&working_dir).unwrap();
    let mut outpaths = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.is_dir() {
            // we don't care about empty dirs
            continue;
        }
        let outpath = match file.enclosed_name() {
            Some(path) => working_dir.join(path),
            None => continue,
        };
        if let Some(p) = outpath.parent() {
            if !p.exists() {
                fs::create_dir_all(p).unwrap();
            }
        }
        let mut outfile = fs::File::create(&outpath).unwrap();
        io::copy(&mut file, &mut outfile).unwrap();

        println!(
            "File extracted to \"{}\" ({} bytes)",
            outpath.display(),
            file.size()
        );

        // get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
        outpaths.push(outpath);
    }
    outpaths
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    #[test]
    fn unzip_returns_correct_number_of_files() {
        // copy zip for use in tests
        let original = "./test-assets/takeout.zip";
        let test_dir = "./test-assets/unzip_returns_correct_number_of_files";
        let test_zip = test_dir.to_string() + ".zip";
        fs::copy(original, &test_zip).unwrap();

        // unzip
        let outpaths = unzip(Path::new(&test_zip));

        // assert
        assert_eq!(outpaths.len(), 8);

        // cleanup
        fs::remove_dir_all(test_dir).unwrap();
        fs::remove_file(test_zip).unwrap();
    }

    #[test]
    fn unzip_keeps_directory_structure() {
        // copy zip for use in tests
        let original = "./test-assets/takeout.zip";
        let test_dir = "./test-assets/unzip_keeps_directory_structure";
        let test_zip = test_dir.to_string() + ".zip";
        fs::copy(original, &test_zip).unwrap();

        // unzip
        unzip(Path::new(&test_zip));

        // assert
        let other_dir = PathBuf::from(test_dir.to_string() + "/takeout/other");
        assert!(other_dir.exists());
        assert!(other_dir.is_dir());

        let other_dir = PathBuf::from(test_dir.to_string() + "/takeout/edited");
        assert!(other_dir.exists());
        assert!(other_dir.is_dir());

        // cleanup
        fs::remove_dir_all(test_dir).unwrap();
        fs::remove_file(test_zip).unwrap();
    }
}

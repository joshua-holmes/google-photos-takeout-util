use std::{collections::VecDeque, fs, io, path::Path};

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

/// Recursively read a given directory and return vector of all file names.
pub fn recursively_collect_filenames(path: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut paths = Vec::new();
    let mut queue = VecDeque::from([path.to_owned()]);
    while let Some(path) = queue.pop_back() {
        for maybe_dir in fs::read_dir(&path)? {
            let dir = maybe_dir?;
            if dir.path().is_file() {
                paths.push(dir.path());
            } else if dir.path().is_dir() {
                queue.push_front(dir.path());
            }
        }
    }

    Ok(paths)
}


#[cfg(test)]
mod tests {
    use std::{collections::HashSet, path::PathBuf};
    use super::*;

    #[test]
    fn collect_filenames_returns_correct_number_of_files() {
        let paths = recursively_collect_filenames(Path::new("./test-assets/takeout-unzipped/")).unwrap();

        assert_eq!(paths.len(), 8);
    }

    #[test]
    fn collect_filenames_returns_valid_paths() {
        let paths = recursively_collect_filenames(Path::new("./test-assets/takeout-unzipped/")).unwrap();

        for p in paths {
            assert!(p.exists());
            assert!(p.is_file());
        }
    }

    #[test]
    fn collect_filenames_returns_same_results_as_unzip() {
        // copy zip for use in tests
        let original = "./test-assets/takeout.zip";
        let test_dir = "./test-assets/collect_filenames_returns_same_results_as_unzip";
        let test_zip = test_dir.to_string() + ".zip";
        fs::copy(original, &test_zip).unwrap();

        let unzip_paths = unzip(Path::new(&test_zip));
        let paths = recursively_collect_filenames(Path::new("./test-assets/takeout-unzipped/")).unwrap();

        // ensure both return same number of files
        assert_eq!(unzip_paths.len(), paths.len());

        // put just the file names in hash sets
        let mut unziped_filenames = HashSet::new();
        let mut filenames = HashSet::new();
        for p in unzip_paths.iter() {
            unziped_filenames.insert(p.file_name().unwrap());
        }
        for p in paths.iter() {
            filenames.insert(p.file_name().unwrap());
        }

        // compare hash sets
        for p in unziped_filenames.iter() {
            println!("Ensuring {:?}, which was unzipped, is in other directory.", p.display());
            assert!(filenames.contains(p));
        }
        for p in filenames.iter() {
            println!("Ensuring {:?}, which is in regular directory, is found in unzipped directory.", p.display());
            assert!(unziped_filenames.contains(p));
        }


        // cleanup
        fs::remove_dir_all(test_dir).unwrap();
        fs::remove_file(test_zip).unwrap();
    }

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

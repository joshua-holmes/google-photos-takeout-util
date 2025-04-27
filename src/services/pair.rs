use std::{collections::{HashMap, HashSet}, fs, io::Read, path::{Path, PathBuf}};


/// A struct for holding the paths of json files and their corresponding images. It is possible for a json file to be
/// linked to up to 2 images, despite the implication of the word "pair".
#[derive(Debug)]
pub struct Pair {
    /// Json file with metadata for corresponding images
    pub json: Option<PathBuf>,
    /// The image associated with the json metadata
    pub img: Option<PathBuf>,
    /// When an image is edited in Google Photos app, it has "-edited" appended to the file name (before the prefix)
    /// and has the same metadata associated with it
    pub img_edited: Option<PathBuf>,
}
impl Pair {
    pub fn new() -> Self {
        Pair {
            json: None,
            img: None,
            img_edited: None,
        }
    }

    /// Read json file, if present. Returns `None` if `self.json` is `None`. Returns error if there was an error
    /// reading the json file.
    pub fn read_json(&self) -> Option<Result<String, PairError>> {
        if let Some(path) = self.json.as_ref() {
            let mut file = fs::File::open(path).unwrap();
            let mut contents = Vec::new();
            if let Err(e) = file.read_to_end(&mut contents) {
                return Some(Err(PairError::IoError(e)));
            }
            match String::from_utf8(contents) {
                Ok(c) => Some(Ok(c)),
                Err(e) => Some(Err(PairError::Utf8ParsingError(e))),
            }
        } else {
            None
        }
    }
}

pub enum PairError {
    IoError(std::io::Error),
    Utf8ParsingError(std::string::FromUtf8Error),
}

pub enum PairComponent {
    Json,
    Img,
    ImgEdited,
}

pub fn create_pairs(set: HashSet<PathBuf>) -> HashMap<String, Pair> {
    let mut pairs = HashMap::new();

    for p in set {
        if p.is_dir() {
            continue;
        }
        let stem = p.file_stem().unwrap().to_str().unwrap();
        let (key, component) = if stem.len() >= 7 && &stem[stem.len() - 7..] == "-edited" {
            let edited_idx = p.to_str().unwrap().find("-edited").unwrap();
            (p.to_str().unwrap()[0..edited_idx].to_string(), PairComponent::ImgEdited)
        } else if p.extension().unwrap().to_str().unwrap() == "json" {
            let img_name = Path::new(p.file_stem().unwrap());
            let naked_key = img_name.file_stem().unwrap();
            let dir = p.parent().unwrap_or(Path::new(""));
            (dir.join(naked_key).to_str().unwrap().to_string(), PairComponent::Json)
        } else {
            let naked_key = p.file_stem().unwrap();
            let dir = p.parent().unwrap_or(Path::new(""));
            (dir.join(naked_key).to_str().unwrap().to_string(), PairComponent::Img)
        };

        let pair = pairs.entry(key).or_insert(Pair::new());
        match component {
            PairComponent::Img => pair.img = Some(p),
            PairComponent::ImgEdited => pair.img_edited = Some(p),
            PairComponent::Json => pair.json = Some(p),
        }
    }

    pairs
}

#[cfg(test)]
mod tests {
    //! Lots of tests because we need to guarantee maximum success when pairing :)

    use super::*;

    #[test]
    fn img_only() {
        let img = PathBuf::from("my_img.jpg");
        let pairs = create_pairs(HashSet::from([
            img.clone(),
        ]));

        let pair = pairs.get("my_img").unwrap();
        assert!(pair.img.is_some());
        assert!(pair.img_edited.is_none());
        assert!(pair.json.is_none());
    }

    #[test]
    fn img_edited_only() {
        let img = PathBuf::from("my_img-edited.jpg");
        let pairs = create_pairs(HashSet::from([
            img.clone(),
        ]));

        let pair = pairs.get("my_img").unwrap();
        assert!(pair.img.is_none());
        assert!(pair.img_edited.is_some());
        assert!(pair.json.is_none());
    }

    #[test]
    fn json_only() {
        let json = PathBuf::from("my_img.jpg.json");
        let pairs = create_pairs(HashSet::from([
            json.clone(),
        ]));

        let pair = pairs.get("my_img").unwrap();
        assert!(pair.img.is_none());
        assert!(pair.img_edited.is_none());
        assert!(pair.json.is_some());
    }

    #[test]
    fn img_only_nested() {
        let img = PathBuf::from("some/dir/my_img.jpg");
        let pairs = create_pairs(HashSet::from([
            img.clone(),
        ]));

        let pair = pairs.get("some/dir/my_img").unwrap();
        assert_eq!(pair.img.as_ref().unwrap(), &img);
        assert!(pair.img_edited.is_none());
        assert!(pair.json.is_none());
    }

    #[test]
    fn img_edited_only_nested() {
        let img = PathBuf::from("some/dir/my_img-edited.jpg");
        let pairs = create_pairs(HashSet::from([
            img.clone(),
        ]));

        let pair = pairs.get("some/dir/my_img").unwrap();
        assert!(pair.img.is_none());
        assert_eq!(pair.img_edited.as_ref().unwrap(), &img);
        assert!(pair.json.is_none());
    }

    #[test]
    fn json_only_nested() {
        let json = PathBuf::from("some/dir/my_img.jpg.json");
        let pairs = create_pairs(HashSet::from([
            json.clone(),
        ]));

        let pair = pairs.get("some/dir/my_img").unwrap();
        assert!(pair.img.is_none());
        assert!(pair.img_edited.is_none());
        assert_eq!(pair.json.as_ref().unwrap(), &json);
    }

    #[test]
    fn img_only_name_with_dots() {
        let img = PathBuf::from("my_img.some.dots.jpg");
        let pairs = create_pairs(HashSet::from([
            img.clone(),
        ]));

        let pair = pairs.get("my_img.some.dots").unwrap();
        assert_eq!(pair.img.as_ref().unwrap(), &img);
        assert!(pair.img_edited.is_none());
        assert!(pair.json.is_none());
    }

    #[test]
    fn img_edited_only_name_with_dots() {
        let img = PathBuf::from("my_img.some.dots-edited.jpg");
        let pairs = create_pairs(HashSet::from([
            img.clone(),
        ]));

        let pair = pairs.get("my_img.some.dots").unwrap();
        assert!(pair.img.is_none());
        assert_eq!(pair.img_edited.as_ref().unwrap(), &img);
        assert!(pair.json.is_none());
    }

    #[test]
    fn json_only_name_with_dots() {
        let json = PathBuf::from("my_img.some.dots.jpg.json");
        let pairs = create_pairs(HashSet::from([
            json.clone(),
        ]));

        let pair = pairs.get("my_img.some.dots").unwrap();
        assert!(pair.img.is_none());
        assert!(pair.img_edited.is_none());
        assert_eq!(pair.json.as_ref().unwrap(), &json);
    }

    #[test]
    fn pair_without_img() {
        let json = PathBuf::from("my_img.jpg.json");
        let img = PathBuf::from("my_img-edited.jpg");
        let pairs = create_pairs(HashSet::from([
            json.clone(),
            img.clone(),
        ]));

        assert_eq!(pairs.len(), 1);

        let pair = pairs.get("my_img").unwrap();
        assert_eq!(pair.img, None);
        assert_eq!(pair.json.as_ref().unwrap(), &json);
        assert_eq!(pair.img_edited.as_ref().unwrap(), &img);
    }

    #[test]
    fn pair_without_img_edited() {
        let json = PathBuf::from("my_img.jpg.json");
        let img = PathBuf::from("my_img.jpg");
        let pairs = create_pairs(HashSet::from([
            json.clone(),
            img.clone(),
        ]));

        assert_eq!(pairs.len(), 1);

        let pair = pairs.get("my_img").unwrap();
        assert_eq!(pair.img.as_ref().unwrap(), &img);
        assert_eq!(pair.json.as_ref().unwrap(), &json);
        assert_eq!(pair.img_edited, None);
    }

    #[test]
    fn pair_without_json() {
        let img = PathBuf::from("my_img.jpg");
        let img_edited = PathBuf::from("my_img-edited.jpg");
        let pairs = create_pairs(HashSet::from([
            img.clone(),
            img_edited.clone(),
        ]));

        assert_eq!(pairs.len(), 1);

        let pair = pairs.get("my_img").unwrap();
        assert_eq!(pair.img.as_ref().unwrap(), &img);
        assert_eq!(pair.json, None);
        assert_eq!(pair.img_edited.as_ref().unwrap(), &img_edited);
    }

    #[test]
    fn pair_with_all_three() {
        let json = PathBuf::from("my_img.jpg.json");
        let img = PathBuf::from("my_img.jpg");
        let img_edited = PathBuf::from("my_img-edited.jpg");
        let pairs = create_pairs(HashSet::from([
            json.clone(),
            img.clone(),
            img_edited.clone(),
        ]));

        assert_eq!(pairs.len(), 1);

        let pair = pairs.get("my_img").unwrap();
        assert_eq!(pair.img.as_ref().unwrap(), &img);
        assert_eq!(pair.json.as_ref().unwrap(), &json);
        assert_eq!(pair.img_edited.as_ref().unwrap(), &img_edited);
    }

    #[test]
    fn multiple_pairs_same_dir() {
        let img1 = PathBuf::from("my_img_a.jpg");
        let img2 = PathBuf::from("my_img_b.jpg");
        let pairs = create_pairs(HashSet::from([
            img1.clone(),
            img2.clone(),
        ]));

        assert_eq!(pairs.len(), 2);

        let pair1 = pairs.get("my_img_a").unwrap();
        let pair2 = pairs.get("my_img_b").unwrap();
        assert_eq!(pair1.img.as_ref().unwrap(), &img1);
        assert_eq!(pair2.img.as_ref().unwrap(), &img2);
    }

    #[test]
    fn multiple_pairs_different_dir() {
        let img1 = PathBuf::from("dir/a/my_img.jpg");
        let img2 = PathBuf::from("dir/b/my_img.jpg");
        let pairs = create_pairs(HashSet::from([
            img1.clone(),
            img2.clone(),
        ]));

        assert_eq!(pairs.len(), 2);

        let pair1 = pairs.get("dir/a/my_img").unwrap();
        let pair2 = pairs.get("dir/b/my_img").unwrap();
        assert_eq!(pair1.img.as_ref().unwrap(), &img1);
        assert_eq!(pair2.img.as_ref().unwrap(), &img2);
    }
}

use std::{collections::HashSet, path::{Iter, Path, PathBuf}};


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
    fn new() -> Self {
        Pair {
            json: None,
            img: None,
            img_edited: None,
        }
    }
}

pub fn create_pairs(mut set: HashSet<PathBuf>) -> Vec<Pair> {
    let mut pairs = Vec::new();

    while let Some(p) = set.iter().next() {
        if p.is_dir() {
            continue;
        }
        let mut pair = Pair::new();
        let stem = p.file_stem().unwrap().to_str().unwrap();
        let (json, img, img_edited) = if stem.len() >= 7 && &stem[stem.len() - 7..] == "-edited" {
            println!("IS EDITED");
            let base = PathBuf::from(p.to_str().unwrap().replace("-edited", ""));
            let json = base.join(".json");
            (json, base, p.clone())
        } else if p.extension().unwrap().to_str().unwrap() == "json" {
            println!("IS JSON");
            let base = PathBuf::from(p.file_stem().unwrap());
            let ext = base.extension().unwrap();
            let edited = PathBuf::from(base.file_stem().unwrap().to_str().unwrap().to_string() + "-edited." + ext.to_str().unwrap());
            (p.clone(), base, edited)
        } else {
            println!("IS BASE");
            let ext = p.extension().unwrap();
            let edited = PathBuf::from(p.file_stem().unwrap().to_str().unwrap().to_string() + "-edited." + ext.to_str().unwrap());
            let json = p.join(".json");
            (json, p.clone(), edited)
        };

        if set.remove(&json) {
            pair.json = Some(json);
        }
        if set.remove(&img) {
            pair.img = Some(img);
        }
        if set.remove(&img_edited) {
            pair.img_edited = Some(img_edited);
        }

        pairs.push(pair);
    }

    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn img_only() {
        let img = PathBuf::from("my_img.jpg");
        let pairs = create_pairs(HashSet::from([
            img.clone(),
        ]));

        let pair = pairs.first().unwrap();
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

        let pair = pairs.first().unwrap();
        assert!(pair.img.is_none());
        assert!(pair.img_edited.is_some());
        assert!(pair.json.is_none());
    }

    #[test]
    fn json_only() {
        let json = PathBuf::from("my_img-edited.jpg.json");
        let pairs = create_pairs(HashSet::from([
            json.clone(),
        ]));

        let pair = pairs.first().unwrap();
        assert!(pair.img.is_none());
        assert!(pair.img_edited.is_none());
        assert!(pair.json.is_some());
    }

    #[test]
    fn basic_pair() {
        let json = PathBuf::from("my_img.jpg.json");
        let img = PathBuf::from("my_img.jpg");
        let pairs = create_pairs(HashSet::from([
            json.clone(),
            img.clone(),
        ]));

        println!("PAIRS {:?}", pairs);

        assert_eq!(pairs.len(), 1);
        let pair = pairs.first().unwrap();

        assert_eq!(pair.img.as_ref().unwrap(), &img);
        assert_eq!(pair.json.as_ref().unwrap(), &json);
        assert_eq!(pair.img_edited, None);
    }

    // #[test]
    fn basic_pair_in_nested_dir() {
        let json = PathBuf::from("some/dir/my_img.jpg.json");
        let img = PathBuf::from("some/dir/my_img.jpg");
        let pairs = create_pairs(HashSet::from([
            json.clone(),
            img.clone(),
        ]));

        assert_eq!(pairs.len(), 1);
        let pair = pairs.first().unwrap();

        assert_eq!(pair.img.as_ref().unwrap(), &img);
        assert_eq!(pair.json.as_ref().unwrap(), &json);
        assert_eq!(pair.img_edited, None);
    }

    fn pair_with_edited_img() {
        let json = PathBuf::from("my_img.jpg.json");
        let img = PathBuf::from("my_img.jpg");
        let img_edited = PathBuf::from("my_img-edited.jpg");
        let pairs = create_pairs(HashSet::from([
            json.clone(),
            img.clone(),
            img_edited.clone(),
        ]));

        assert_eq!(pairs.len(), 1);
        let pair = pairs.first().unwrap();

        assert_eq!(pair.img.as_ref().unwrap(), &img);
        assert_eq!(pair.json.as_ref().unwrap(), &json);
        assert_eq!(pair.img_edited.as_ref().unwrap(), &img_edited);
    }
}

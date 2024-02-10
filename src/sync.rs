use crate::sphere::ReadParseFile;
use crate::sphere::Sphere;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::usize;
const MAX_SENTENCE: usize = 300;
struct App;
impl App {
    fn compose(conn: &mut Spoken) {
        for Lang { data, file_name } in &conn.release {
            let mut list = vec![];
            for (rank, mas) in data {
                if conn.keys.contains(rank) {
                    list.push((rank, mas))
                }
            }
            list.sort_by(|(a, _), (b, _)| a.cmp(b));
            let mut table = HashMap::new();
            let mut content = String::new();
            for (rank, inner_mas) in list {
                let line = &format!("{}, {}, {}\n", rank, inner_mas.word, inner_mas.sentence);
                if inner_mas.word != "w" {
                    if let Some(start) = table.get_mut(&inner_mas.word) {
                        content.push_str(&format!(
                            "{}, {}-{}, {}\n",
                            rank, inner_mas.word, start, inner_mas.sentence
                        ));
                        *start += 1;
                    } else {
                        content.push_str(line);
                        table.insert(inner_mas.word.clone(), 1);
                    }
                } else {
                    content.push_str(line);
                }
            }
            File::write(&conn.folder_name, file_name, content).unwrap();
        }
    }

    fn process(conn: &mut Spoken) {
        let mut english = Lang::new(&conn.folder_name, conn.store.len(), &conn.english_path);
        let mut new_english = Lang::new_clean(conn.store.len(), &conn.english_path);
        let mut changes = vec![];
        for (old_rank, mas) in &english.data.clone() {
            conn.store.get(&mas.word).map(|new_rank| {
                let node = english.data.remove(old_rank).unwrap();
                if new_rank != old_rank {
                    new_english.data.insert(*new_rank, node).unwrap();
                    changes.push(Change::new(*new_rank, *old_rank));
                } else {
                    new_english.data.insert(*old_rank, node).unwrap();
                }
            });
        }

        for (word, rank) in &conn.store {
            new_english.data.get_mut(rank).map(|mas| {
                if mas.word != *word {
                    mas.word = word.clone();
                }
            });
        }
        conn.release.push(new_english);

        for file_name in &conn.files_names {
            let mut lang = Lang::new(&conn.folder_name, conn.store.len(), file_name);
            let original = lang.data.clone();
            let mut empty = Lang::new_clean(conn.store.len(), file_name);
            let mut do_not_update = HashSet::new();
            for Change { new, old } in &changes {
                let mut new_mas = lang.data.remove(old).unwrap();
                new_mas.rank = *new;

                empty.data.insert(*new, new_mas);
                do_not_update.insert(new);
            }

            for key in lang
                .data
                .clone()
                .keys()
                .filter(|rank| !do_not_update.contains(rank))
            {
                lang.data.remove(key).map(|mas| {
                    empty.data.insert(*key, mas);
                });
            }
            for Change { new, old } in &changes {
                let temp = empty.data.clone();
                let left = temp.get(new).unwrap();
                let right = original.get(old).unwrap();
                if left.word != right.word {
                    empty.data.insert(*new, Mas::new(*new, "w", "s")).unwrap();
                }
            }
            conn.release.push(empty);
        }
        App::compose(conn);
    }
    fn start(conn: &mut Spoken) {
        for (index, word) in conn.list.iter().enumerate() {
            let rank = index + 1;
            conn.keys.insert(rank);
            conn.store.insert(word.to_owned(), rank);
        }

        App::process(conn)
    }
}

#[derive(Debug)]
struct Spoken {
    list: Vec<String>,
    files_names: Vec<String>,
    store: HashMap<String, usize>,
    release: Vec<Lang>,
    keys: HashSet<usize>,
    folder_name: String,
    english_path: String,
}

impl Spoken {
    fn new(file: File, tipo: &String, sphere: &Sphere) -> Self {
        Self {
            list: sphere
                .vocabulary
                .data
                .get(tipo)
                .map_or(&vec![], |inner| inner)
                .iter()
                .map(|mas| mas.word.to_string())
                .collect(),
            files_names: file.files_names,
            store: HashMap::new(),
            release: vec![],
            keys: HashSet::new(),
            folder_name: file.folder_name.to_owned(),
            english_path: "english.on".to_string(),
        }
    }
}

#[derive(Debug)]
struct Change {
    new: usize,
    old: usize,
}
impl Change {
    fn new(new: usize, old: usize) -> Self {
        Self { new, old }
    }
}
#[derive(Debug)]
struct Lang {
    file_name: String,
    data: HashMap<usize, Mas>,
}
impl Lang {
    fn new(folder_name: &str, max: usize, file_name: &str) -> Self {
        let mut error_line = 1;
        let mut data = HashMap::new();
        let path = format!("{}/{}", folder_name, file_name);
        let content = fs::read_to_string(&path).unwrap();
        for line in File::parse_into_lines(content) {
            let mut mas = File::parse_line(&line, &path, error_line);
            if mas.word.contains("-") {
                let mio: Vec<_> = mas.word.split("-").collect();
                mas.word = mio[0].to_owned();
            }
            data.insert(mas.rank, mas);
            error_line += 1;
        }

        for rank in 1..=max {
            if data.get(&rank).is_none() {
                data.insert(rank, Mas::new(rank, "w", "s"));
            }
        }

        Self {
            data,
            file_name: file_name.to_owned(),
        }
    }
    fn new_clean(max: usize, file_name: &str) -> Self {
        let mut data = HashMap::new();
        for rank in 1..=max {
            data.insert(rank, Mas::new(rank, "w", "s"));
        }
        Self {
            data,
            file_name: file_name.to_owned(),
        }
    }
}
#[derive(Debug, Clone)]
struct Mas {
    rank: usize,
    word: String,
    sentence: String,
}
impl Mas {
    fn new(rank: usize, word: &str, sentence: &str) -> Self {
        Self {
            rank,
            word: word.to_owned(),
            sentence: sentence.to_owned(),
        }
    }
}

struct Line {}
impl Line {
    fn sentence(s: &str) -> String {
        let list: Vec<_> = s
            .split_whitespace()
            .filter_map(|piece| {
                let piece = piece.trim();
                if piece == "" {
                    None
                } else {
                    Some(piece)
                }
            })
            .collect();
        list.join(" ")
    }
    fn word(w: &str) -> &str {
        w.split_whitespace().next().unwrap().trim()
    }
}

#[derive(Clone, Debug)]
struct File {
    folder_name: String,
    files_names: Vec<String>,
    file_path: String,
}
impl File {
    fn new() -> Self {
        Self {
            folder_name: String::new(),
            files_names: Vec::new(),
            file_path: String::new(),
        }
    }

    fn read(path: &str) -> String {
        fs::read_to_string(path).unwrap()
    }

    fn write(folder_name: &str, file_name: &str, content: String) -> std::io::Result<()> {
        fs::write(format!("{}/{}", folder_name, file_name), content)?;
        Ok(())
    }
    fn parse_into_lines(content: String) -> Vec<String> {
        let list: Vec<_> = content
            .split("\n")
            .map(|n| n.trim())
            .filter(|n| n.len() > 0)
            .map(|n| n.to_owned())
            .collect();
        list
    }

    fn parse_line(line: &str, file_path: &str, file_line: usize) -> Mas {
        let list: Vec<&str> = line
            .split(",")
            .map(|item| item.trim())
            .filter(|item| item != &"")
            .collect();

        assert!(
            list.len() == 3,
            "Error in file {} line {} contains {} extras commas only 3 are allow in each line (rank, word, sentence) ",
            file_path,
            file_line,
            list.len() - 3
        );
        let num: usize = list[0].parse().unwrap();
        let sentence = &Line::sentence(list[2]);
        assert!(
            sentence.len() <= MAX_SENTENCE,
            "Error in file {} line {} contains {} characters the maximum allow are {}  Please remove {} characters",
            file_path,
            file_line,
            sentence.len(),
            MAX_SENTENCE,
             sentence.len() -  MAX_SENTENCE

        );
        Mas::new(num, Line::word(list[1]), sentence)
    }
    fn parse_line_release(line: &str) -> Mas {
        let list: Vec<&str> = line
            .split(",")
            .map(|item| item.trim())
            .filter(|item| item != &"")
            .collect();

        if list.len() != 3 {
            panic!(
                "Each files must contains (rank,  word, sentence) in order to process the content"
            )
        }
        let num: usize = list[0].parse().unwrap();
        Mas::new(num, Line::word(list[1]), &Line::sentence(list[2]))
    }
    fn setup(mut self, folder: &String, connector: &Sync) -> Self {
        self.folder_name = format!("spoken/{}", folder);
        self.files_names = connector
            .languages
            .folders
            .iter()
            .filter(|lang| lang != &"english")
            .map(|lang| format!("{}.on", lang))
            .collect();
        self.file_path = format!("{}/english.on", self.folder_name);

        self
    }
}
pub struct Release;
impl Release {
    pub fn write(connector: &Sync, sphere: &Sphere, report: &mut Vec<String>) {
        for folder in &sphere.config.types.list {
            for language in &connector.languages.folders {
                let name = format!("{}.on", language);
                let file_name = format!("spoken/{}/{}", folder, name);
                let content = File::read(&file_name);
                let mut size = 0;
                let new_content = File::parse_into_lines(content)
                    .into_iter()
                    .map(|line| File::parse_line_release(&line))
                    .filter(|mas| mas.word != "w" && mas.sentence != "s")
                    .fold(String::new(), |mut acc, mas| {
                        let new_line = format!("{},{},{}\n", mas.rank, mas.word, mas.sentence);
                        acc.push_str(&new_line);
                        size += 1;
                        acc
                    });

                let type_folder = &format!("release/{}", &folder);
                let inner_folder = Path::new(type_folder);

                if new_content.len() > 0 {
                    let file_report = format!("{}, {}, {}", folder, name, size);
                    report.push(file_report);
                    if !inner_folder.exists() {
                        fs::create_dir(&inner_folder).unwrap();
                    }
                    File::write(&type_folder, &name, new_content).unwrap();
                }
            }
        }
    }

    pub fn write_report(list: Vec<String>) {
        fs::write("release/report.on", list.join("\n")).unwrap();
    }
}
pub fn inner(connector: &Sync, sphere: &Sphere) {
    println!("\nSYNC Running...");
    for folder in &sphere.config.types.list {
        let file = File::new().setup(folder, &connector);
        App::start(&mut Spoken::new(file, folder, &sphere));
    }
}

#[derive(Clone, Debug)]
pub struct Carrier {
    pub dir: PathBuf,
    pub folder: String,
    pub types: Vec<String>,
    pub paths: Vec<PathBuf>,
}
impl Carrier {
    fn new() -> Self {
        Self {
            dir: PathBuf::new(),
            folder: String::new(),
            types: Vec::new(),
            paths: Vec::new(),
        }
    }
    fn build(mut self, folder: &String, types: &Vec<String>, dir: &PathBuf) -> Self {
        self.folder = folder.to_owned();
        self.types = types.clone();
        self.dir = dir.clone();
        self
    }
}
#[derive(Clone)]
pub struct Languages {
    pub dir: PathBuf,
    pub folders: Vec<String>,
    pub carrier: Vec<Carrier>,
}

impl Languages {
    fn new() -> Self {
        Self {
            dir: PathBuf::new(),
            folders: Vec::new(),
            carrier: Vec::new(),
        }
    }

    fn put_carrier(&self, types: &Vec<String>) -> Vec<Carrier> {
        let mut acc = vec![];
        for folder in types {
            let dir = self.dir.join(folder);
            if !dir.is_dir() {
                fs::create_dir(&dir).unwrap();
            }

            let mut carrier = Carrier::new().build(folder, types, &dir);

            for tipo in &self.folders {
                let name = format!("{}.on", tipo);
                let path = dir.join(&name);
                if !path.exists() {
                    fs::File::create(&path).unwrap();
                }
                carrier.paths.push(path);
            }

            acc.push(carrier);
        }
        acc
    }

    fn setup(mut self, connector: &Sync, sphere: &Sphere) -> Self {
        self.dir = connector.dir.join("spoken");
        self.folders = ReadParseFile::new("config/languages.on").split_whitespace();
        self.carrier = self.put_carrier(&sphere.config.types.list);
        self
    }
}

pub struct Sync {
    pub languages: Languages,
    pub dir: PathBuf,
}
impl Sync {
    pub fn new() -> Self {
        Self {
            languages: Languages::new(),
            dir: PathBuf::new(),
        }
    }

    pub fn setup(mut self, sphere: &Sphere) -> Self {
        self.dir = env::current_dir().unwrap();
        self.languages = self.languages.clone().setup(&self, &sphere);
        self
    }

    pub fn start() {
        let sphere = Sphere::new().setup();
        let connector = Sync::new().setup(&sphere);
        inner(&connector, &sphere);
    }
}

pub mod utils {
    use chrono::{TimeZone, Utc};
    use polars::prelude::*;
    use soup::{NodeExt, QueryBuilderExt, Soup};
    use std::{
        collections::HashMap,
        fmt::{self},
        fs::{self, File, OpenOptions},
        io::{BufRead, BufReader, Read, Write},
    };

    fn request(url: &str) -> Option<reqwest::blocking::Response> {
        let res = reqwest::blocking::get(url).expect("Errore nella get");

        if res.status().is_success() {
            Some(res)
        } else {
            None
        }
    }

    enum Type {
        DocumentClass,
        Package,
    }

    /// controlla quando è stato eseguito l'ultimo aggiornamento dei file
    #[allow(unused_must_use)]
    fn get_last(r#type: Type) -> bool {
        let name = format!(
            "./history/history-{}.txt",
            match r#type {
                Type::DocumentClass => "class",
                Type::Package => "package",
            }
        );
        let path = std::path::Path::new(&name);
        let now = Utc::now();

        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap());
            fs::File::create(path);
            let mut file = OpenOptions::new().write(true).open(path).unwrap();
            writeln!(&mut file, "{}", now.format("%d/%m/%Y"));
            drop(file);
            return false;
        }

        let file = File::open(path).unwrap();
        let last: Vec<u32> = BufReader::new(file)
            .lines()
            .last()
            .unwrap()
            .unwrap()
            .split("/")
            .map(|x| x.parse::<u32>().unwrap())
            .collect();

        let last_get = Utc
            .with_ymd_and_hms(last[2] as i32, last[1], last[0], 0, 0, 0)
            .unwrap();

        let diff = (now - last_get).num_days();
        if diff > 90 {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(path)
                .unwrap();
            writeln!(&mut file, "{}", now.format("%d/%m/%Y"));
            return true;
        }

        false
    }

    #[derive(Debug)]
    pub struct DocumentClass(DataFrame);

    impl fmt::Display for DocumentClass {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl DocumentClass {
        const PATH: &'static str = "./data/documentClass.csv";

        pub fn new() -> Self {
            if super::utils::get_last(super::utils::Type::DocumentClass) {
                let mut class = Self::create();
                class.to_csv();
                return class;
            }

            if std::path::Path::new(Self::PATH).exists() {
                Self::read_csv()
            } else {
                let mut class = Self::create();
                class.to_csv();
                super::utils::get_last(super::utils::Type::DocumentClass);
                class
            }
        }

        /// Prende il risultato di una get e estrapolo i nomi delle classi dal body
        fn create() -> Self {
            let mut res = request("https://ctan.org/topic/class").unwrap();
            let mut body = String::new();
            res.read_to_string(&mut body)
                .expect("Errore nella lettura della request");

            let div = Soup::new(&body)
                .tag("div")
                .attr_value("dt")
                .find_all()
                .collect::<Vec<_>>();

            let mut document_class = Vec::new();

            div.into_iter().for_each(|ele| {
                document_class.extend(
                    ele.children()
                        .filter(|node| node.is_element())
                        .map(|node| node.text().to_string())
                        .collect::<Vec<_>>(),
                );
            });

            // Create dataframe
            Self(
                DataFrame::new(vec![
                    Series::new(
                        "index",
                        (1..=document_class.len() as i32).collect::<Vec<_>>(),
                    ),
                    Series::new("documentClass", document_class),
                ])
                .expect("Errore nella creazione del Dataframe"),
            )
        }

        /// Salvo il dataframe su un file csv
        #[allow(unused_must_use)]
        fn to_csv(&mut self) {
            if !std::path::Path::new(Self::PATH).exists() {
                fs::create_dir_all("./data").expect("Errore nella creazione della directory");
            }

            let mut file = File::create(Self::PATH).expect("Errore nell'apertura del file");

            CsvWriter::new(&mut file)
                .include_header(true)
                .with_separator(b',')
                .finish(&mut self.0);
        }

        /// Leggo un file csv e creo il dataframe
        fn read_csv() -> Self {
            Self(
                CsvReader::from_path("./data/documentClass.csv")
                    .expect("Errore nell'apertura del file")
                    .has_header(true)
                    .finish()
                    .expect("Errore nella creazione del DataFrame"),
            )
        }

        /// Numero di tutte le classi
        pub fn len(&self) -> usize {
            self.0.height()
        }
    }

    pub struct Package(pub HashMap<String, DataFrame>);

    impl Package {
        pub fn new() -> Self {
            if super::utils::get_last(super::utils::Type::Package) {
                let mut package = Self::create();
                package.to_csv();
                return package;
            }

            if std::path::Path::new("./data/packages").exists() {
                Self::read_csv()
            } else {
                let mut package = Self::create();
                package.to_csv();
                super::utils::get_last(super::utils::Type::Package);
                package
            }
        }

        /// Esegue una request e colleziona il value dei tag dt in vettore
        fn collect_data(url: &str) -> Vec<String> {
            let mut res = request(url).unwrap();
            let mut body = String::new();
            res.read_to_string(&mut body)
                .expect("Errore nella lettura del body ");

            let div = Soup::new(&body)
                .tag("div")
                .attr_value("dt")
                .find_all()
                .collect::<Vec<_>>();

            let mut data = Vec::new();

            div.into_iter().for_each(|ele| {
                data.extend(
                    ele.children()
                        .filter(|node| node.is_element())
                        .map(|node| node.text().to_string())
                        .collect::<Vec<_>>(),
                );
            });
            data
        }

        fn create() -> Self {
            let mut packages = HashMap::<String, DataFrame>::new();
            let package_obsolete = Package::collect_data("https://www.ctan.org/topic/obsolete");
            let class = Package::collect_data("https://ctan.org/topic/class");

            for char in 'A'..='Z' {
                let url = format!("https://ctan.org/pkg/:{}", char);
                let package_raw = Package::collect_data(url.as_str());

                // rimuovo i pacchetti che sono o obsoleti o delle classi
                let package: Vec<String> = package_raw
                    .iter()
                    .filter(|x| !package_obsolete.contains(x))
                    .filter(|x| !class.contains(x))
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();

                packages.insert(
                    char.to_string(),
                    DataFrame::new(vec![
                        Series::new("index", (1..=package.len() as i32).collect::<Vec<_>>()),
                        Series::new("packages", package),
                    ])
                    .expect("Errore nella creazione del dataframe"),
                );
            }

            Self(packages)
        }

        #[allow(unused_must_use)]
        pub(crate) fn to_csv(&mut self) {
            let path = "./data/packages";

            if !std::path::Path::new(path).exists() {
                fs::create_dir_all(path).expect("Errore nella creazione della directory");
            }

            for element in &self.0 {
                let path_inner = format!("{}/{}", path, element.0);
                fs::create_dir(&path_inner).expect("Errore nella creazione della cartella");
                let file = match File::create(format!("{}/package-{}.csv", path_inner, element.0)) {
                    Ok(file) => Some(file),
                    Err(_) => None,
                };

                if file.is_none() {
                    fs::remove_dir_all(path);
                    println!("Errore nel salvataggio dei pacchetti");
                    return;
                }

                CsvWriter::new(&mut file.unwrap())
                    .include_header(true)
                    .with_separator(b',')
                    .finish(&mut self.0.get(element.0).unwrap().clone());
            }
        }

        fn read_csv() -> Self {
            let mut package = HashMap::<String, DataFrame>::new();
            let path = "./data/packages";

            for char in 'A'..='Z' {
                let filename = format!("{}/{}/package-{}.csv", path, char, char);

                let el = CsvReader::from_path(filename)
                    .expect("Errore nell'apertura del file")
                    .has_header(true)
                    .finish()
                    .expect("Errore nella creazione del DataFrame");

                package.insert(char.to_string(), el);
            }
            Self(package)
        }

        /// Numero di tutti i pacchetti
        pub fn len(&self) -> usize {
            let mut size = 0;
            for data in self.0.values() {
                size += data.height();
            }
            size
        }
    }

    impl fmt::Debug for Package {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for char in 'A'..='Z' {
                writeln!(f, "{}: {:?}\n", char, self.0.get(char.to_string().as_str()))?;
            }
            Ok(())
        }
    }
}

pub mod lib {
    use polars::prelude::*;
    use soup::{NodeExt, QueryBuilderExt, Soup};
    use std::collections::HashMap;
    use std::fmt::{self};
    use std::{
        fs::{self, File},
        io::Read,
    };

    pub fn request(url: &str) -> Option<reqwest::blocking::Response> {
        let res = reqwest::blocking::get(url).expect("Errore nella get");

        if res.status().is_success() {
            Some(res)
        } else {
            None
        }
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
            if std::path::Path::new(Self::PATH).exists() {
                Self::read_csv()
            } else {
                let mut class = Self::create();
                class.to_csv();
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

    pub struct Package(HashMap<String, DataFrame>);

    impl Package {
        pub fn new() -> Self {
            if std::path::Path::new("./data/packages").exists() {
                Self::read_csv()
            } else {
                let mut class = Self::create();
                class.to_csv();
                class
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
            let mut package = HashMap::<String, DataFrame>::new();

            for char in 'A'..='Z' {
                let url = format!("https://ctan.org/pkg/:{}", char);
                let package_raw = Package::collect_data(url.as_str());
                let package_obsolete = Package::collect_data("https://www.ctan.org/topic/obsolete");
                let class = Package::collect_data("https://ctan.org/topic/class");

                let pack_clear: Vec<String> = package_raw
                    .iter()
                    .filter(|x| !package_obsolete.contains(x))
                    .filter(|x| !class.contains(x))
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();

                package.insert(
                    char.to_string(),
                    DataFrame::new(vec![
                        Series::new("index", (1..=pack_clear.len() as i32).collect::<Vec<_>>()),
                        Series::new("packages", pack_clear),
                    ])
                    .expect("Errore nella creazione del dataframe"),
                );
            }
            Self(package)
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

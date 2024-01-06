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

    #[derive(Debug, Clone)]
    pub struct DocumentClass(DataFrame);

    impl fmt::Display for DocumentClass {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl DocumentClass {
        pub fn new() -> Self {
            unimplemented!()
        }

        /// Prende il risultato di una get e estrapolo i nomi delle classi dal body
        pub fn create() -> Self {
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
        pub fn to_csv(&mut self) {
            let path = "./data/documentClass.csv";

            if !std::path::Path::new(path).exists() {
                fs::create_dir_all("./data").expect("Errore nella creazione della directory");
            }

            let mut file = File::create(path).expect("Errore nell'apertura del file");

            CsvWriter::new(&mut file)
                .include_header(true)
                .with_separator(b',')
                .finish(&mut self.0);
        }

        /// Leggo un file csv e creo il dataframe
        pub fn read_csv() -> Self {
            Self(
                CsvReader::from_path("./data/documentClass.csv")
                    .expect("Errore nell'apertura del file")
                    .has_header(true)
                    .finish()
                    .expect("Errore nella creazione del DataFrame"),
            )
        }

        pub fn len(&self) -> usize {
            self.0.height()
        }
    }

    pub struct Package(HashMap<String, DataFrame>);

    impl Package {
        pub fn new() -> Self {
            unimplemented!()
        }

        pub fn create() -> Self {
            let mut package = HashMap::<String, DataFrame>::new();

            for char in 'A'..='Z' {
                let url = format!("https://ctan.org/pkg/:{}", char);
                let mut res = request(url.as_str()).unwrap();
                let mut body = String::new();
                res.read_to_string(&mut body)
                    .expect("Errore nella lettura del body ");

                let div = Soup::new(&body)
                    .tag("div")
                    .attr_value("dt")
                    .find_all()
                    .collect::<Vec<_>>();

                let mut tmp_pack = Vec::new();

                div.into_iter().for_each(|ele| {
                    tmp_pack.extend(
                        ele.children()
                            .filter(|node| node.is_element())
                            .map(|node| node.text().to_string())
                            .collect::<Vec<_>>(),
                    );
                });

                package.insert(
                    char.to_string(),
                    DataFrame::new(vec![
                        Series::new("index", (1..=tmp_pack.len() as i32).collect::<Vec<_>>()),
                        Series::new("documentClass", tmp_pack),
                    ])
                    .expect("Errore nella creazione del dataframe"),
                );
            }
            Self(package)
        }

        #[allow(unused_must_use)]
        pub fn to_csv(&mut self) {
            let path = "./data/packages";

            if !std::path::Path::new(path).exists() {
                fs::create_dir_all(path).expect("Errore nella creazione della directory");
            }

            for element in &self.0 {
                let path_inner = format!("{}/{}", path, element.0);
                fs::create_dir(&path_inner).expect("Errore nella creazione della cartella");
                let mut file = File::create(format!("{}/package-{}.csv", path_inner, element.0))
                    .expect("Errore nella creazione del file");

                CsvWriter::new(&mut file)
                    .include_header(true)
                    .with_separator(b',')
                    .finish(&mut self.0.get(element.0).unwrap().clone());
            }
        }

        pub fn read_csv() -> Self {
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

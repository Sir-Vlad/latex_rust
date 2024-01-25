pub mod document {
    use core::fmt;
    use std::{
        fs::{self, File, OpenOptions},
        io::Write,
        path::Path,
    };

    use askama::Template;
    use chrono::{DateTime, Utc};

    #[derive(Debug, Template)]
    #[template(path = "template_latex.txt", ext = "txt")]
    pub struct FileLatex<'a> {
        document_class: &'a str,
        packages: MultiOpzioni<'a>, // todo: da modificare quando implemento le opzione per ogni pacchetto
        title: &'a str,
        authors: MultiOpzioni<'a>,
        pub date: DateTime<Utc>,
    }

    impl<'a> FileLatex<'a> {
        pub fn new(
            document_class: &'a str,
            packages: MultiOpzioni<'a>,
            title: &'a str,
            authors: MultiOpzioni<'a>,
            date: DateTime<Utc>,
        ) -> Self {
            Self {
                document_class,
                packages,
                title,
                authors,
                date,
            }
        }

        fn today(date: DateTime<Utc>) -> bool {
            Utc::now().date_naive().eq(&date.date_naive())
        }
    }

    #[derive(Debug)]
    pub struct MultiOpzioni<'a>(Option<Vec<&'a str>>);

    impl<'a> MultiOpzioni<'a> {
        pub fn new(opzioni: Option<Vec<&'a str>>) -> Self {
            Self(opzioni)
        }
    }

    impl<'a> fmt::Display for MultiOpzioni<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match &self.0 {
                Some(v) => write!(f, "{}", v.join(", ")),
                None => Ok(()),
            }
        }
    }

    #[derive(Debug)]
    pub struct ProjectLatex<'a> {
        path: Box<&'a Path>,
        nome: &'a str,
        content: FileLatex<'a>,
    }

    impl<'a> ProjectLatex<'a> {
        pub fn new(path: Box<&'a Path>, nome: &'a str, content: FileLatex<'a>) -> Self {
            Self {
                path,
                nome,
                content,
            }
        }
    }

    #[allow(unused_must_use)]
    pub fn create_document(file: ProjectLatex) {
        let path_project = format!("{}/{}", file.path.display(), file.nome);
        // create workspace
        match fs::create_dir_all(&path_project) {
            Ok(_) => (),
            Err(why) => panic!("Could not create directory {:?}: {}", path_project, why),
        }
        // create work folder
        for el in &["grafici", "capitoli", "immagini"] {
            fs::create_dir(format!("{}/{}", path_project, el));
        }

        // create file command.sty e main.tex
        File::create(format!("{}/command.sty", path_project));
        let ref path_main = format!("{}/main.tex", path_project);
        File::create(path_main);
        let mut main = OpenOptions::new()
            .write(true)
            .open(Path::new(path_main))
            .unwrap();

        writeln!(&mut main, "{}", file.content.render().unwrap()).expect("");
    }
}

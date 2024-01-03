// use core::fmt;

// use chrono::NaiveDate;

fn main() {
    // let name_folder = input("Inserisci il nome del folder");
    // let name_document_class = input("Inserisci il nome della classe del documento");
    // let title_document = input("Inserisci il titolo del documento");
    // let authors = split_authors(input(
    //     "Inserisci il/i nomi degli autori (separati da virgola)",
    // ));

    // let date_document;
    // loop {
    //     match create_date(input("Inserisci la data del documento (dd/mm/yyyy)")) {
    //         Some(d) => {
    //             date_document = d;
    //             break;
    //         }
    //         None => println!("Data non valida!!!"),
    //     }
    // }
    // print!("{}", date_document)

    run();
}

fn run(){
    let response = 
}


// fn create_date(input: String) -> Option<NaiveDate> {
//     let input_split: Vec<_> = input.trim().split("/").collect();

//     let list_date: Vec<_> = input_split
//         .into_iter()
//         .map(|x| x.trim().parse::<u32>().unwrap())
//         .collect();

//     return NaiveDate::from_ymd_opt(list_date[2] as i32, list_date[1], list_date[1]);
// }

// /// Input da terminale
// fn input(msg: &str) -> String {
//     let mut line = String::new();
//     println!("{msg}");
//     std::io::stdin().read_line(&mut line).unwrap();
//     line.trim().to_string()
// }

// #[derive(Debug)]
// enum StringOrList {
//     String(String),
//     List(Vec<String>),
// }

// impl StringOrList {
//     /// Ritorna la lunghezza
//     /// - se è una stringa ritorna il numero di caratteri
//     /// - se è una lista ritorna il numero di elementi
//     fn len(&self) -> usize {
//         match self {
//             StringOrList::List(x) => x.len(),
//             StringOrList::String(x) => x.len(),
//         }
//     }
// }

// impl fmt::Display for StringOrList {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             StringOrList::String(_) => write!(f, "{}", self),
//             StringOrList::List(lista) => {
//                 let mut tmp = String::new();
//                 tmp.push_str("[");
//                 lista.into_iter().for_each(|el| {
//                     tmp.push_str(format!("{} ", el).as_str());
//                 });
//                 tmp.remove(tmp.len() - 1); // delete last space
//                 tmp.push_str("]");
//                 write!(f, "{}", tmp)
//             }
//         }
//     }
// }

// fn split_authors(text: String) -> StringOrList {
//     if text.contains(',') {
//         StringOrList::List(
//             text.trim()
//                 .split(",")
//                 .map(|x| x.trim().to_string())
//                 .filter(|x| x.len() > 1)
//                 .collect(),
//         )
//     } else {
//         StringOrList::String(text)
//     }
// }

// #[cfg(test)]
// #[path = "./test/test.rs"]
// mod test;

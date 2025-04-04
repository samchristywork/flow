use clap::Parser;
use regex::Regex;
use std::{collections::HashSet, fs::read_to_string, process::exit};

#[derive(Clone)]
struct Function {
    name: String,
    body: String,
    module: String,
}

impl Function {
    const fn new() -> Self {
        Self {
            name: String::new(),
            body: String::new(),
            module: String::new(),
        }
    }

    fn body_length(&self) -> usize {
        self.body.lines().count()
    }

    fn label(&self) -> String {
        format!("{}:{}", self.name, self.body_length())
    }

    fn check_for_function_in_body(&self, f: &Self) -> bool {
        // TODO: Add better error message
        Regex::new(&format!(r"\b{}\b\(", f.name))
            .unwrap()
            .is_match(&self.body)
    }

    fn link(&self, f: &Self) -> String {
        format!(r#""{}"->"{}";"#, self.label(), f.label())
    }
}

fn extract_function_name(line: &str, re: &Regex) -> String {
    re.find(line).map_or_else(
        || line.to_string(),
        |m| extract_function_name(&(String::new() + &line[..m.start()] + &line[m.end()..]), re),
    )
}

fn extract_functions(
    filenames: &[String],
    start: &Regex,
    end: &Regex,
    function_cleanup: &Regex,
) -> Vec<Function> {
    filenames
        .iter()
        .fold(Vec::new(), |mut functions, filename| {
            // TODO: Do this in main
            let source_code = read_to_string(filename).unwrap();
            let mut function = Function::new();
            let loc = source_code.lines().count();

            for line in source_code.lines() {
                // Handle the start of a function
                if start.is_match(line) {
                    function.name = extract_function_name(line, function_cleanup);
                    function.module = format!("{filename}:{loc}"); // TODO
                    function.body = line.to_string();

                // Handle the body of the function
                } else if !function.name.is_empty() {
                    function.body.push_str((String::from(line) + "\n").as_str());

                    // Recognize the end of a function
                    if end.is_match(line) {
                        functions.push(function);
                        function = Function::new();
                    }
                }
            }
            functions
        })
}

fn generate_cluster(module: &str, functions: &[Function]) -> String {
    "subgraph cluster_".to_string()
        + &module.replace(['.', '-', '/', ':'], "_")
        + "{label=\""
        + module
        + "\";bgcolor=\"#eeeeee\";"
        + &functions
            .iter()
            .filter(|f| f.module == *module)
            .fold(String::new(), |acc, f| acc + "\"" + &f.label() + "\";")
        + "}"
}

fn generate_clusters(functions: &[Function]) -> String {
    functions
        .iter()
        .map(|f| f.module.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .iter()
        .map(|module| generate_cluster(module, functions))
        .collect::<String>()
}

fn generate_links(functions: &[Function]) -> String {
    functions
        .iter()
        .map(|f1| {
            functions
                .iter()
                .filter(|f2| f1.check_for_function_in_body(f2))
                .map(|f2| f1.link(f2))
                .collect::<String>()
        })
        .collect::<String>()
}

fn table_row(label: &str, value: &str) -> String {
    format!("<tr><td align=\"left\">{label}</td><td align=\"right\">{value}</td></tr>")
}

fn generate_legend(functions: &[Function], modules: &[String]) -> String {
    let lines_of_code = functions.iter().map(Function::body_length).sum::<usize>();
    let num_functions = functions.len();
    let num_modules = modules.len();

    "legend".to_string()
        + "["
        + "shape=plaintext;"
        + "label=<"
        + "<table border=\"0\" cellborder=\"1\" cellspacing=\"0\">"
        + &table_row("Modules", format!("{num_modules}").as_str())
        + &table_row("Functions", format!("{num_functions}").as_str())
        + &table_row("Lines", format!("{lines_of_code}").as_str())
        + "</table>"
        + ">;"
        + "];"
}

fn generate_callgraph(
    filenames: &[String],
    start: &Regex,
    end: &Regex,
    function_cleanup: &Regex,
) -> String {
    let functions = extract_functions(filenames, start, end, function_cleanup);
    let modules = functions
        .iter()
        .map(|f| f.module.clone())
        .collect::<HashSet<_>>();

    String::from("strict digraph {")
        + "graph [rankdir=LR];"
        + "node [shape=box;style=filled;fillcolor=\"#ffffff\"];"
        + generate_legend(&functions, &modules.into_iter().collect::<Vec<_>>()).as_str()
        + generate_clusters(&functions).as_str()
        + generate_links(&functions).as_str()
        + "}"
}

#[derive(Parser, Debug)]
#[clap(author = "Sam Christy", version = "1.0", about = "Generate a callgraph from source code", long_about = None)]
struct Args {
    /// Regex pattern to match the start of a function
    #[clap(short, long, default_value = r"^fn |^pub fn ")]
    start: String,

    /// Regex pattern to match the end of a function
    #[clap(short, long, default_value = r"^}$")]
    end: String,

    /// Regex pattern to match parts of the function signature to ignore
    #[clap(short, long, default_value = r"\w+ |\(.+|\(")]
    function_blacklist: String,

    /// Input files
    #[clap(value_parser)]
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let (start, end, function_cleanup) = match (
        Regex::new(&args.start),
        Regex::new(&args.end),
        Regex::new(&args.function_blacklist),
    ) {
        (Ok(s), Ok(e), Ok(f)) => (s, e, f),
        (Err(err), _, _) | (_, Err(err), _) | (_, _, Err(err)) => {
            eprintln!("Error: {err}");
            exit(1);
        }
    };

    let callgraph = generate_callgraph(&args.files, &start, &end, &function_cleanup);
    println!("{callgraph}");
}

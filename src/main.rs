use clap::Parser;
use regex::Regex;
use std::{fs::read_to_string, process::exit};

#[derive(Clone)]
struct Module {
    filename: String,
    source: String,
}

impl Module {
    fn label(&self) -> String {
        format!("{}:{}", self.filename, self.source.lines().count())
    }

    fn id(&self) -> String {
        self.filename.chars().fold(String::new(), |acc, c| {
            if c.is_alphanumeric() {
                acc + &c.to_string()
            } else {
                acc + "_"
            }
        })
    }
}

#[derive(Clone)]
struct Function {
    name: String,
    body: String,
    module: Module,
}

impl Function {
    fn label(&self) -> String {
        format!("{}:{}", self.name, self.body.lines().count())
    }

    fn check_for_function_in_body(&self, f: &Self) -> bool {
        let body = self.body.lines().skip(1).collect::<String>();
        let pattern = format!(r"\b{}\b", f.name);
        Regex::new(&pattern).map_or_else(
            |_| {
                eprintln!(
                    "Error: Invalid regex pattern\nFunction: {}\nPattern: {pattern}",
                    f.name
                );
                exit(1);
            },
            |re| re.is_match(&body),
        )
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
    modules: &[Module],
    start: &Regex,
    end: &Regex,
    function_cleanup: &Regex,
    ignore_function: &Regex,
) -> Vec<Function> {
    modules.iter().fold(Vec::new(), |mut functions, module| {
        let mut function = Function {
            name: String::new(),
            body: String::new(),
            module: module.clone(),
        };
        for line in module.source.lines() {
            // Handle the start of a function
            if start.is_match(line) {
                function.name = extract_function_name(line, function_cleanup);
                function.module = module.clone();
                function.body = line.to_string() + "\n";

            // Handle the body of the function
            } else if !function.name.is_empty() {
                function.body.push_str((String::from(line) + "\n").as_str());

                // Recognize the end of a function
                if end.is_match(line) {
                    if !ignore_function.is_match(&function.name) {
                        functions.push(function);
                    }
                    function = Function {
                        name: String::new(),
                        body: String::new(),
                        module: module.clone(),
                    };
                }
            }
        }

        if !function.name.is_empty() {
            if !ignore_function.is_match(&function.name) {
                functions.push(function);
            }
        }

        functions
    })
}

fn generate_cluster(module: &Module, functions: &[Function]) -> String {
    "subgraph cluster_".to_string()
        + &module.id()
        + "{label=\""
        + &module.label()
        + "\";bgcolor=\"#eeeeee\";"
        + &functions
            .iter()
            .filter(|f| f.module.filename == module.filename)
            .fold(String::new(), |acc, f| acc + "\"" + &f.label() + "\";")
        + "}"
}

fn generate_clusters(modules: &[Module], functions: &[Function]) -> String {
    modules
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

fn generate_legend(modules: &[Module], functions: &[Function]) -> String {
    let lines_of_code = modules
        .iter()
        .map(|m| m.source.lines().count())
        .sum::<usize>();
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
    modules: &[Module],
    start: &Regex,
    end: &Regex,
    function_cleanup: &Regex,
    ignore_function: &Regex,
) -> String {
    let functions = extract_functions(modules, start, end, function_cleanup, ignore_function);

    modules.iter().for_each(|m| eprintln!("Module: {}", m.filename));
    functions.iter().for_each(|f| eprintln!("Function: {}", f.name));

    String::from("strict digraph {")
        + "graph [rankdir=LR];"
        + "node [shape=box;style=filled;fillcolor=\"#ffffff\"];"
        + generate_legend(modules, &functions).as_str()
        + generate_clusters(modules, &functions).as_str()
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
    #[clap(short, long, default_value = r"\(.+|\($|^\w+ ")]
    function_blacklist: String,

    /// Regex pattern to match functions to ignore
    #[clap(short, long, default_value = r"^$")]
    ignore_function: String,

    /// Input files
    #[clap(value_parser)]
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let (start, end, function_cleanup, ignore_function) = match (
        Regex::new(&args.start),
        Regex::new(&args.end),
        Regex::new(&args.function_blacklist),
        Regex::new(&args.ignore_function),

    ) {
        (Ok(s), Ok(e), Ok(f), Ok(i)) => (s, e, f, i),
        (Err(err), _, _, _) | (_, Err(err), _, _) | (_, _, Err(err), _) | (_, _, _, Err(err)) => {
            eprintln!("Error: {err}");
            exit(1);
        }
    };

    let modules: Vec<Module> = args
        .files
        .iter()
        .map(|filename| Module {
            filename: filename.clone(),
            source: read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Error: Could not read file {filename}");
                exit(1);
            }),
        })
        .collect();

    let callgraph = generate_callgraph(&modules, &start, &end, &function_cleanup, &ignore_function);
    println!("{callgraph}");
}

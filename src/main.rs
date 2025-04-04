use regex::Regex;

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
        regex::Regex::new(&format!(r"\b{}\b\(", f.name))
            .expect("Invalid regex pattern")
            .is_match(&self.body)
    }

    fn link(&self, f: &Self) -> String {
        format!(r#""{}" -> "{}";"#, self.label(), f.label())
    }
}

fn extract_function_name(line: &str) -> String {
    let idx = line.find('(').expect("Failed to find function identifier");

    (*line[..idx]
        .split_whitespace()
        .collect::<Vec<_>>()
        .last()
        .expect("Failed to find function identifier"))
    .to_string()
}

fn extract_functions(
    filenames: &[String],
    start: &Regex,
    end: &Regex,
) -> Result<Vec<Function>, String> {
    let mut functions = Vec::new();
    for filename in filenames {
        let source_code = std::fs::read_to_string(filename)
            .map_err(|e| format!("Failed to read file {filename}: {e}"))?;
        let mut function = Function::new();
        let loc = source_code.lines().count();

        for line in source_code.lines() {
            // Handle the start of a function
            if start.is_match(line) {
                function.name = extract_function_name(line);
                function.module = format!("{filename}:{loc}");
                function.body = String::from("{");
                continue;
            }

            // Handle the body of the function
            if !function.name.is_empty() {
                function.body.push_str(format!("{line}\n").as_str());

                // Recognize the end of a function
                if end.is_match(line) {
                    functions.push(function);
                    function = Function::new();
                }
            }
        }
    }

    Ok(functions)
}

fn generate_cluster(module: &str, functions: &[Function]) -> String {
    "subgraph cluster_".to_string()
        + &module.replace(['.', '-', '/', ':'], "_")
        + "{label = \""
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
        .collect::<std::collections::HashSet<_>>()
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

fn generate_legend(functions: &[Function]) -> String {
    let lines_of_code = functions.iter().map(|f| f.body_length()).sum::<usize>();
    let num_functions = functions.len();
    let num_modules = functions
        .iter()
        .map(|f| f.module.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .len();

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

fn generate_callgraph(filenames: &[String], start: &Regex, end: &Regex) -> Result<String, String> {
    let functions = extract_functions(filenames, start, end)?;
    Ok(String::from("strict digraph {")
        + "graph [rankdir=LR];"
        + "node [shape=box;style=filled;fillcolor=\"#ffffff\"];"
        + generate_legend(&functions).as_str()
        + generate_clusters(&functions).as_str()
        + generate_links(&functions).as_str()
        + "}")
}

fn main() {
    let filenames = std::env::args().skip(1).collect::<Vec<_>>();
    let start = Regex::new(r"^fn |^pub fn ").expect("Invalid regex pattern");
    let end = Regex::new(r"^}").expect("Invalid regex pattern");

    let callgraph = generate_callgraph(&filenames, &start, &end).unwrap_or_else(|err| {
        eprintln!("Error: {err}");
        std::process::exit(1);
    });
    println!("{callgraph}");
}

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

fn extract_functions(filenames: &[String], start: &str, end: &str) -> Vec<Function> {
    filenames
        .iter()
        .fold(Vec::new(), |mut functions, filename| {
            let source_code = std::fs::read_to_string(filename)
                .unwrap_or_else(|_| panic!("Failed to read file: {filename}"));

            let mut function = Function::new();
            for line in source_code.lines() {
                if line.starts_with(start) {
                    function = Function {
                        name: line[3..line
                            .find('(')
                            .expect("No opening parenthesis found in the function signature")]
                            .to_string(),
                        body: String::from("{"),
                        module: String::from(filename),
                    };
                    continue;
                }

                if !function.name.is_empty() {
                    function.body.push_str(line);
                    function.body.push('\n');

                    if line.starts_with(end) {
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
        + &module.replace(['.', '-', '/'], "_")
        + "{label = \""
        + module
        + "\";"
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

fn generate_callgraph(filenames: &[String], start: &str, end: &str) -> String {
    let functions = extract_functions(filenames, start, end);
    String::from("strict digraph {graph [rankdir=LR];node [shape=box];")
        + generate_clusters(&functions).as_str()
        + generate_links(&functions).as_str()
        + "}"
}

fn main() {
    let filenames = std::env::args().skip(1).collect::<Vec<_>>();
    println!("{}", generate_callgraph(&filenames, "fn ", "}"));
}

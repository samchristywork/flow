#[derive(Clone)]
struct Function {
    name: String,
    body: String,
    module: String,
}

impl Function {
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

    fn clear(&mut self) {
        self.name.clear();
        self.body.clear();
        self.module.clear();
    }

    fn link(&self, f: &Self) -> String {
        format!(r#""{}" -> "{}";"#, self.label(), f.label())
    }
}

fn extract_functions(filenames: &[String], start: &str, end: &str) -> Vec<Function> {
    let mut functions = Vec::new();

    for filename in filenames {
        let source_code = std::fs::read_to_string(filename)
            .unwrap_or_else(|_| panic!("Failed to read file: {filename}"));

        let mut function = Function {
            name: String::new(),
            body: String::new(),
            module: String::new(),
        };

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
                    functions.push(function.clone());
                    function.clear();
                }
            }
        }
    }

    functions
}

fn sanitize_filename(filename: &str) -> String {
    filename.replace(['.', '-', '/'], "_")
}

fn generate_callgraph(filenames: &[String], start: &str, end: &str) -> String {
    let functions = extract_functions(filenames, start, end);
    let modules = functions
        .iter()
        .map(|f| f.module.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    String::from("strict digraph {graph [rankdir=LR];node [shape=box];")
        + &modules
            .iter()
            .map(|module| {
                "subgraph cluster_".to_string()
                    + &sanitize_filename(module)
                    + "{label = \""
                    + module
                    + "\";"
                    + &functions
                        .iter()
                        .filter(|f| f.module == *module)
                        .fold(String::new(), |acc, f| acc + "\"" + &f.label() + "\";")
                    + "}"
            })
            .collect::<String>()
        + &functions
            .iter()
            .map(|f1| {
                functions
                    .iter()
                    .filter(|f2| f1.check_for_function_in_body(f2))
                    .map(|f2| f1.link(f2))
                    .collect::<String>()
            })
            .collect::<String>()
        + "}"
}

fn main() {
    let filenames = std::env::args().skip(1).collect::<Vec<_>>();
    println!("{}", generate_callgraph(&filenames, "fn ", "}"));
}

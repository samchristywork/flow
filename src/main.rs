#[derive(Clone)]
struct Function {
    name: String,
    body: String,
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
    }

    fn link(&self, f: &Self) -> String {
        format!(r#""{}" -> "{}";"#, self.label(), f.label())
    }
}

fn extract_functions(source_code: &str, start: &str, end: &str) -> Vec<Function> {
    let mut function = Function {
        name: String::new(),
        body: String::new(),
    };

    let mut functions = Vec::new();
    for line in source_code.lines() {
        if line.starts_with(start) {
            function = Function {
                name: line[3..line
                    .find('(')
                    .expect("No opening parenthesis found in the function signature")]
                    .to_string(),
                body: String::from("{"),
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

    functions
}

fn generate_callgraph(filename: &str, start: &str, end: &str) -> String {
    let source_code = std::fs::read_to_string(filename).expect("Unable to read file");
    let functions = extract_functions(&source_code, start, end);
    String::from("strict digraph {graph [rankdir=LR];node [shape=box];")
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
    let filename = std::env::args().nth(1).expect("No filename provided");
    println!("{}", generate_callgraph(&filename, "fn ", "}"));
}

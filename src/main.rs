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
}

fn extract_functions(source_code: &str, start_pattern: &str, end_pattern: &str) -> Vec<Function> {
    let mut function = Function {
        name: String::new(),
        body: String::new(),
    };

    let mut functions = Vec::new();
    for line in source_code.lines() {
        if line.starts_with(start_pattern) {
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

            if line.starts_with(end_pattern) {
                functions.push(function.clone());
                function.clear();
            }
        }
    }

    functions
}

fn main() {
}

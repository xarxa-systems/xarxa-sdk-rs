
type Step = (String, Box<dyn Fn() -> Result<(), String> + 'static>);
pub struct Pipeline {
    steps: Vec<Step>,
}

enum StepType {
    Action(String),
}

impl Pipeline {
    pub fn new() -> Pipeline {
        Pipeline {
            steps: Vec::new(),
        }
    }

    pub fn step<F>(mut self, name: &str, f: F) -> Self
    where
        F: Fn() -> Result<(), String> + 'static,
    {
        self.steps.push((name.to_string(), Box::new(f)));
        self
    }

    pub fn run(self) -> Result<(), String> {
        for (name, step) in self.steps {
            println!("Executing step: {name}");
            
            step()?;
        }

        Ok(())
    }
}

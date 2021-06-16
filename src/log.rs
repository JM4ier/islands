use std::io::*;
use std::time::*;

pub struct Logger {
    start: SystemTime,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            start: SystemTime::now(),
        }
    }
    pub fn do_task<T>(&self, title: &str, fun: impl FnOnce() -> T) -> T {
        print!("{: <25}", title);
        stdout().lock().flush().ok();
        let start = std::time::SystemTime::now();
        let result = fun();
        match start.elapsed() {
            Ok(elapsed) => println!("({:.3} s)", elapsed.as_secs_f32()),
            Err(e) => eprintln!("(Timing error: {:?})", e),
        }
        result
    }
    pub fn finalize(self) {
        println!(
            "{: <25}[{:.3} s]\n",
            "Total",
            self.start.elapsed().expect("timing error").as_secs_f32()
        );
    }
}

use configurations::Configurations;

mod cleaner;
mod configurations;

fn main() {
    cleaner::clean(&Configurations::new());
}

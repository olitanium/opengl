use opengl::{environment::Environment, Result};

mod state;

fn main() -> Result<()> {
    let mut environment = Environment::<state::State>::new(
        (3, 3),
        (1920, 1080),
        "Window",
        !cfg!(debug_assertions),
    )?;

    environment.run()?;

    Ok(())
}
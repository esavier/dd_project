use std::error::Error;
use vergen_gix::GixBuilder;
use vergen::{BuildBuilder, CargoBuilder, Emitter, RustcBuilder, SysinfoBuilder};


fn main() -> Result<(), Box<dyn Error>> {
    // Emit the instructions
    let build = BuildBuilder::default().build_timestamp(true).build()?;
    let cargo = CargoBuilder::default().opt_level(true).build()?;
    let git = GixBuilder::default().commit_timestamp(true).build()?;
    let rustc = RustcBuilder::default().semver(true).build()?;
    let si = SysinfoBuilder::default().cpu_core_count(true).build()?;
    
    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&git)?
        .add_instructions(&rustc)?
        .add_instructions(&si)?
        .emit()?;

    Ok(())
}

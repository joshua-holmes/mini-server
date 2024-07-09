use crate::types;

pub fn handle(odin_body: types::valheim::OdinBody) -> Result<(), ()> {
    println!("BODY {:?}", odin_body);
    Ok(())
}

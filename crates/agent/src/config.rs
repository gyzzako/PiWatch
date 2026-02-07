use std::{fs, path::Path, io::Result};

const NODE_ID_DIR: &'static str = "/var/lib/PiWatch/";

pub(crate) fn load_or_create_node_id() -> Result<String> {
    let node_id_file = format!("{}node_id", NODE_ID_DIR);

    if Path::new(&node_id_file).exists() {
        Ok(fs::read_to_string(&node_id_file)?.trim().to_string())
    } else {
        fs::create_dir_all(NODE_ID_DIR)?;
        let id = uuid::Uuid::new_v4().to_string();
        fs::write(&node_id_file, &id)?;
        Ok(id)
    }
}
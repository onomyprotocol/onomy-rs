use std::{collections::BTreeMap, process::Command};

pub fn stop_containers(active_container_ids: &mut BTreeMap<String, String>) {
    for (name, id) in active_container_ids.iter() {
        let rm_output = Command::new("docker").args(["rm", id]).output().unwrap();
        if rm_output.status.success() {
            println!("stopped container {}", name);
        } else {
            println!("tried to stop container {} and got {:?}", name, rm_output);
        }
    }
    active_container_ids.clear();
}

pub fn force_stop_containers(active_container_ids: &mut BTreeMap<String, String>) {
    for (name, id) in active_container_ids.iter() {
        let rm_output = Command::new("docker")
            .args(["rm", "-f", id])
            .output()
            .unwrap();
        if rm_output.status.success() {
            println!("force stopped container {}", name);
        } else {
            println!(
                "tried to force stop container {} and got {:?}",
                name, rm_output
            );
        }
    }
    active_container_ids.clear();
}

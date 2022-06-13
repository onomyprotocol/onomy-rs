//! `docker-compose` is not programmable among other problems. What this does is
//! create an `--internal` network of containers using locally built binaries.
//! Logs from `stdout` and `stderr` are actively pushed to log files under
//! `testcrate/logs`. Most failure cases are all caught and all containers are
//! force stopped.

use std::{collections::BTreeMap, fs, io, path::PathBuf};

use common::{
    command::{assert_dir_exists, assert_file_exists, run_command, run_command_with_output},
    docker::force_stop_containers,
};

struct Args {
    dir: String,
    network: String,
    target: String,
}

#[tokio::main]
async fn main() {
    let args = Args {
        dir: "./".to_owned(),
        network: "testnet".to_owned(),
        target: "x86_64-unknown-linux-gnu".to_owned(),
    };
    // check the directory for expected folders we will be using
    let base_dir = PathBuf::from(&args.dir);
    assert_dir_exists(&base_dir).unwrap();
    let base_dir = fs::canonicalize(base_dir).unwrap();
    let base_dir_s = base_dir.to_str().unwrap();
    println!("using base directory {}", base_dir_s);
    assert!(base_dir_s.ends_with("onomy-rs"));

    let log_dir = base_dir.join("testcrate/logs");
    assert_dir_exists(&log_dir).unwrap();

    println!("running `cargo build --release --target {}`", args.target);
    run_command(
        "cargo",
        &["build", "--release", "--target", &args.target],
        Some(&log_dir.join("cmd_cargo_build_out.log")),
        Some(&log_dir.join("cmd_cargo_build_err.log")),
    )
    .await
    .unwrap();

    let bin_dir = base_dir.join(format!("target/{}/release", args.target));
    assert_dir_exists(&bin_dir).unwrap();

    let containers = [("host_equity", "equity-core")];
    for (_, bin) in &containers {
        assert_file_exists(&bin_dir.join(bin)).unwrap();
    }

    // create an `--internal` docker network
    println!("creating docker network {}", args.network);
    // remove old network if it exists
    run_command(
        "docker",
        &["network", "rm", &args.network],
        Some(&log_dir.join("cmd_docker_network_rm_out.log")),
        Some(&log_dir.join("cmd_docker_network_rm_err.log")),
    )
    .await
    .unwrap();
    // create the network as `--internal`
    run_command(
        "docker",
        &["network", "create", "--internal", &args.network],
        Some(&log_dir.join("cmd_docker_network_create_out.log")),
        Some(&log_dir.join("cmd_docker_network_create_err.log")),
    )
    .await
    .unwrap();

    let base_image = "fedora:36".to_owned();

    // run all the creation first so that everything is pulled and prepared
    let mut active_container_ids: BTreeMap<String, String> = BTreeMap::new();
    for (container_name, bin) in &containers {
        let bin_path = bin_dir.join(bin);
        let bin_path_s = bin_path.to_str().unwrap();
        // just include the needed binary
        let volume = format!("{}:/usr/bin/{}", bin_path_s, &bin);
        let args = vec![
            "create",
            "--rm",
            "--network",
            &args.network,
            "--hostname",
            container_name,
            "--name",
            container_name,
            "--volume",
            &volume,
            &base_image,
            bin,
        ];
        match run_command_with_output(
            "docker",
            &args,
            Some(&log_dir.join("cmd_docker_create_err.log")),
        )
        .await
        {
            Ok(mut id) => {
                // remove trailing '\n'
                id.pop().unwrap();
                println!("started container {}", container_name);
                active_container_ids.insert(container_name.to_string(), id);
            }
            Err(e) => {
                println!("force stopping all containers: {}\n", e);
                force_stop_containers(&mut active_container_ids);
                panic!();
            }
        }
    }

    // start all containers
    for (_, id) in active_container_ids.clone() {
        let args = vec!["start", &id];
        match run_command_with_output(
            "docker",
            &args,
            Some(&log_dir.join("cmd_docker_start_err.log")),
        )
        .await
        {
            Ok(name) => {
                println!("started container {}", name)
            }
            Err(e) => {
                println!("force stopping all containers: {}\n", e);
                force_stop_containers(&mut active_container_ids);
                panic!();
            }
        }
    }

    println!("press enter to stop");
    let _ = io::stdin().read_line(&mut String::new());

    force_stop_containers(&mut active_container_ids);
}

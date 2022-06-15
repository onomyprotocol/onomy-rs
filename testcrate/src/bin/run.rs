//! `docker-compose` is not programmable among other problems. What this does is
//! create an `--internal` network of containers using locally built binaries.
//! Logs from `stdout` and `stderr` are actively pushed to log files under
//! `testcrate/logs`. Most failure cases are all caught and all containers are
//! force stopped on failure or finish.
//!
//! The overall run waits on the last container in the list finishing

use std::{collections::BTreeMap, fs, path::PathBuf};

use common::{
    command::{assert_dir_exists, assert_file_exists, ComplexCommand},
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
    ComplexCommand::new("cargo", &["build", "--release", "--target", &args.target])
        .unwrap()
        .stderr_to_file(&log_dir.join("cmd_cargo_build_err.log"))
        .await
        .unwrap()
        .wait()
        .await
        .unwrap();

    // after the build we should have a release directory with the binaries
    let bin_dir = base_dir.join(format!("target/{}/release", args.target));
    assert_dir_exists(&bin_dir).unwrap();

    // TODO need some kind of system for multiple test groups

    // name followed by the binary used
    let containers = [
        ("equity_core", "equity_core"),
        ("test_health", "test_health"),
    ];
    if containers.is_empty() {
        panic!();
    }
    for (_, bin) in &containers {
        assert_file_exists(&bin_dir.join(bin)).unwrap();
    }

    // create an `--internal` docker network
    println!("creating docker network {}", args.network);
    // remove old network if it exists (there is no option to ignore nonexistent
    // networks, drop exit status errors)
    let _ = ComplexCommand::new("docker", &["network", "rm", &args.network])
        .unwrap()
        .wait()
        .await;
    // create the network as `--internal`
    ComplexCommand::new("docker", &[
        "network",
        "create",
        "--internal",
        &args.network,
    ])
    .unwrap()
    .wait_for_output()
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
        match ComplexCommand::new("docker", &args)
            .unwrap()
            .stderr_to_file(&log_dir.join("cmd_docker_create_err.log"))
            .await
            .unwrap()
            .wait_for_output()
            .await
        {
            Ok(output) => {
                let mut id = output.stdout;
                // remove trailing '\n'
                id.pop().unwrap();
                println!("created container {}", container_name);
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
    let mut ccs = vec![];
    for (container_name, id) in active_container_ids.clone().iter() {
        let args = vec!["start", "--attach", id];
        let stderr = log_dir.join(format!("container_{}_err.log", container_name));
        let cc = ComplexCommand::new("docker", &args)
            .unwrap()
            .stderr_to_file(&stderr)
            .await
            .unwrap();
        ccs.push(cc);
    }

    let cc = ccs.pop().unwrap();
    // wait on last container finishing
    print!("waiting on last container... ",);
    match cc.wait().await {
        Ok(()) => {
            println!("done");
        }
        Err(e) => {
            println!("force stopping all containers: {}\n", e);
            force_stop_containers(&mut active_container_ids);
            panic!();
        }
    }

    //println!("press enter to stop");
    //let _ = io::stdin().read_line(&mut String::new());

    force_stop_containers(&mut active_container_ids);
}

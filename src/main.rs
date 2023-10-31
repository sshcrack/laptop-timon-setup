use anyhow::Result;
use nix::unistd::Uid;
use std::{
    env::{self, current_exe},
    fs::{self, copy, Permissions},
    os::unix::prelude::PermissionsExt,
    path::PathBuf,
    process::Command,
    thread,
    time::Duration,
};

const STR_DAEMON: &str = "/root/wireguard-daemon";
const STR_SERVICE: &str = "/etc/systemd/system/wireguard-checker.service";
fn main() -> Result<()> {
    if !Uid::effective().is_root() {
        panic!("You must run this executable with root permissions");
    }

    let args: Vec<String> = env::args().collect();
    let run_daemon = args.len() > 1 && args[1] == "--daemon";
    let is_uninstall = args.len() > 1 && args[1] == "uninstall";
    if is_uninstall {
        println!("Uninstall");
        uninstall()?;
        return Ok(());
    }

    if !run_daemon {
        setup()?;
        println!("Setup done.");
        return Ok(());
    }

    let stat = wg_helper("up");
    if let Err(x) = stat {
        eprintln!("Could not start vpn: {:?}. Probably already started.", x);
    }
    loop {
        println!("Pinging...");
        let mut ping_cmd = Command::new("ping")
            .arg("1.1.1.1")
            .arg("-c")
            .arg("4")
            .spawn()?;

        let res = ping_cmd.wait();
        let is_err = res.is_err() || res.as_ref().is_ok_and(|e| !e.success());

        if is_err {
            let err = res.as_ref().err();
            let res = res.as_ref().ok();
            println!(
                "Detected ping error: {:?}, {:?}. Reconnecting VPN...",
                err, res
            );
            let res = wg_helper("down");
            if let Err(x) = res {
                eprintln!("Could not stop VPN: {:?}", x);
            }
            let res = wg_helper("up");
            if let Err(z) = res {
                eprintln!("Could not start VPN: {:?}", z);
            }
        }
        println!("Waiting 60 seconds...");
        thread::sleep(Duration::from_secs(60));
    }
}

fn wg_helper(stat: &str) -> Result<()> {
    println!("Running wg-quick {} wg0...", stat);
    Command::new("wg-quick").arg(stat).arg("wg0").output()?;

    Ok(())
}

fn uninstall() -> Result<()> {
    println!("Stopping service...");
    let out = Command::new("systemctl")
        .arg("stop")
        .arg("wireguard-checker.service")
        .output()?;

    println!("{:?}", out);

    println!("Disabling service...");
    let out = Command::new("systemctl")
        .arg("disable")
        .arg("wireguard-checker.service")
        .output()?;

    println!("{:?}", out);

    println!("Removing daemon...");
    fs::remove_file(&STR_DAEMON)?;

    println!("Removing service...");
    fs::remove_file(&STR_SERVICE)?;

    Ok(())
}

fn setup() -> Result<()> {
    let exe = current_exe()?;
    let out_exe = PathBuf::from(STR_DAEMON);

    println!("Copying daemon...");
    copy(exe, &out_exe)?;
    fs::set_permissions(out_exe, Permissions::from_mode(0o700))?;

    let service = include_str!("./daemon.service").replace("EXEC_DAEMON", STR_DAEMON);

    let service_path = PathBuf::from(STR_SERVICE);

    println!("Writing service...");
    fs::write(service_path, service)?;

    println!("Enabling service...");
    let out = Command::new("systemctl")
        .arg("enable")
        .arg("wireguard-checker.service")
        .output()?;

    println!("{:?}", out);
    println!("Starting service...");

    let out = Command::new("systemctl")
        .arg("start")
        .arg("wireguard-checker.service")
        .output()?;

    println!("{:?}", out);

    Ok(())
}

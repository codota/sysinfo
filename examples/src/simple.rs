//
// Sysinfo
//
// Copyright (c) 2017 Guillaume Gomez
//

#![crate_type = "bin"]
#![allow(unused_must_use, non_upper_case_globals)]

extern crate sysinfo;

use std::io::{self, BufRead, Write};
use std::str::FromStr;
use sysinfo::Signal::*;
use sysinfo::{
    NetworkExt, NetworksExt, Pid, ProcessExt, ProcessorExt, Signal, System, SystemExt, UserExt,
};

const signals: [Signal; 31] = [
    Hangup,
    Interrupt,
    Quit,
    Illegal,
    Trap,
    Abort,
    Bus,
    FloatingPointException,
    Kill,
    User1,
    Segv,
    User2,
    Pipe,
    Alarm,
    Term,
    Stklft,
    Child,
    Continue,
    Stop,
    TSTP,
    TTIN,
    TTOU,
    Urgent,
    XCPU,
    XFSZ,
    VirtualAlarm,
    Profiling,
    Winch,
    IO,
    Power,
    Sys,
];

fn print_help() {
    writeln!(&mut io::stdout(), "== Help menu ==");
    writeln!(&mut io::stdout(), "help               : show this menu");
    writeln!(
        &mut io::stdout(),
        "signals            : show the available signals"
    );
    writeln!(
        &mut io::stdout(),
        "refresh            : reloads all processes' information"
    );
    writeln!(
        &mut io::stdout(),
        "refresh [pid]      : reloads corresponding process' information"
    );
    writeln!(
        &mut io::stdout(),
        "refresh_users      : reloads only users' information"
    );
    writeln!(
        &mut io::stdout(),
        "show [pid | name]  : show information of the given process \
         corresponding to [pid | name]"
    );
    writeln!(
        &mut io::stdout(),
        "kill [pid] [signal]: send [signal] to the process with this \
         [pid]. 0 < [signal] < 32"
    );
    writeln!(
        &mut io::stdout(),
        "processors         : Displays processors state"
    );
    writeln!(
        &mut io::stdout(),
        "memory             : Displays memory state"
    );
    writeln!(
        &mut io::stdout(),
        "temperature        : Displays components' temperature"
    );
    writeln!(
        &mut io::stdout(),
        "network            : Displays network' information"
    );
    writeln!(
        &mut io::stdout(),
        "all                : Displays all process name and pid"
    );
    writeln!(
        &mut io::stdout(),
        "uptime             : Displays system uptime"
    );
    writeln!(
        &mut io::stdout(),
        "boot_time          : Displays system boot time"
    );
    writeln!(
        &mut io::stdout(),
        "vendor_id          : Displays processor vendor id"
    );
    writeln!(
        &mut io::stdout(),
        "brand              : Displays processor brand"
    );
    writeln!(
        &mut io::stdout(),
        "load_avg           : Displays system load average"
    );
    writeln!(
        &mut io::stdout(),
        "frequency          : Displays processor frequency"
    );
    writeln!(&mut io::stdout(), "users              : Displays all users");
    writeln!(&mut io::stdout(), "quit               : exit the program");
}

fn interpret_input(input: &str, sys: &mut System) -> bool {
    match input.trim() {
        "help" => print_help(),
        "refresh_users" => {
            writeln!(&mut io::stdout(), "Refreshing user list...");
            sys.refresh_users_list();
            writeln!(&mut io::stdout(), "Done.");
        }
        "signals" => {
            let mut nb = 1i32;

            for sig in &signals {
                writeln!(&mut io::stdout(), "{:2}:{:?}", nb, sig);
                nb += 1;
            }
        }
        "processors" => {
            // Note: you should refresh a few times before using this, so that usage statistics
            // can be ascertained
            writeln!(
                &mut io::stdout(),
                "total process usage: {}%",
                sys.get_global_processor_info().get_cpu_usage()
            );
            for proc_ in sys.get_processors() {
                writeln!(&mut io::stdout(), "{:?}", proc_);
            }
        }
        "memory" => {
            writeln!(
                &mut io::stdout(),
                "total memory: {} KB",
                sys.get_total_memory()
            );
            writeln!(
                &mut io::stdout(),
                "used memory : {} KB",
                sys.get_used_memory()
            );
            writeln!(
                &mut io::stdout(),
                "total swap  : {} KB",
                sys.get_total_swap()
            );
            writeln!(
                &mut io::stdout(),
                "used swap   : {} KB",
                sys.get_used_swap()
            );
        }
        "quit" | "exit" => return true,
        "all" => {
            for (pid, proc_) in sys.get_processes() {
                writeln!(
                    &mut io::stdout(),
                    "{}:{} status={:?}",
                    pid,
                    proc_.name(),
                    proc_.status()
                );
            }
        }
        "frequency" => {
            writeln!(
                &mut io::stdout(),
                "{} MHz",
                sys.get_processors()[0].get_frequency()
            );
        }
        "vendor_id" => {
            writeln!(
                &mut io::stdout(),
                "vendor ID: {}",
                sys.get_processors()[0].get_vendor_id()
            );
        }
        "brand" => {
            writeln!(
                &mut io::stdout(),
                "brand: {}",
                sys.get_processors()[0].get_brand()
            );
        }
        "load_avg" => {
            let load_avg = sys.get_load_average();
            writeln!(&mut io::stdout(), "one minute     : {}%", load_avg.one);
            writeln!(&mut io::stdout(), "five minutes   : {}%", load_avg.five);
            writeln!(&mut io::stdout(), "fifteen minutes: {}%", load_avg.fifteen);
        }
        e if e.starts_with("show ") => {
            let tmp: Vec<&str> = e.split(' ').collect();

            if tmp.len() != 2 {
                writeln!(
                    &mut io::stdout(),
                    "show command takes a pid or a name in parameter!"
                );
                writeln!(&mut io::stdout(), "example: show 1254");
            } else if let Ok(pid) = Pid::from_str(tmp[1]) {
                match sys.get_process(pid) {
                    Some(p) => writeln!(&mut io::stdout(), "{:?}", *p),
                    None => writeln!(&mut io::stdout(), "pid \"{:?}\" not found", pid),
                };
            } else {
                let proc_name = tmp[1];
                for proc_ in sys.get_process_by_name(proc_name) {
                    writeln!(&mut io::stdout(), "==== {} ====", proc_.name());
                    writeln!(&mut io::stdout(), "{:?}", proc_);
                }
            }
        }
        "temperature" => {
            for component in sys.get_components() {
                writeln!(&mut io::stdout(), "{:?}", component);
            }
        }
        "network" => {
            for (interface_name, data) in sys.get_networks().iter() {
                writeln!(
                    &mut io::stdout(),
                    "{}:\n  input data  (new / total): {} / {} B\n  output data (new / total): {} / {} B",
                    interface_name,
                    data.get_received(),
                    data.get_total_received(),
                    data.get_transmitted(),
                    data.get_total_transmitted(),
                );
            }
        }
        "show" => {
            writeln!(
                &mut io::stdout(),
                "'show' command expects a pid number or a process name"
            );
        }
        e if e.starts_with("kill ") => {
            let tmp: Vec<&str> = e.split(' ').collect();

            if tmp.len() != 3 {
                writeln!(
                    &mut io::stdout(),
                    "kill command takes the pid and a signal number in parameter !"
                );
                writeln!(&mut io::stdout(), "example: kill 1254 9");
            } else {
                let pid = Pid::from_str(tmp[1]).unwrap();
                let signal = i32::from_str(tmp[2]).unwrap();

                if signal < 1 || signal > 31 {
                    writeln!(
                        &mut io::stdout(),
                        "Signal must be between 0 and 32 ! See the signals list with the \
                         signals command"
                    );
                } else {
                    match sys.get_process(pid) {
                        Some(p) => {
                            writeln!(
                                &mut io::stdout(),
                                "kill: {}",
                                p.kill(*signals.get(signal as usize - 1).unwrap())
                            );
                        }
                        None => {
                            writeln!(&mut io::stdout(), "pid not found");
                        }
                    };
                }
            }
        }
        "users" => {
            for user in sys.get_users() {
                writeln!(&mut io::stdout(), "{:?}", user.get_name());
            }
        }
        "boot_time" => {
            writeln!(&mut io::stdout(), "{} seconds", sys.get_boot_time());
        }
        "uptime" => {
            let up = sys.get_uptime();
            let mut uptime = sys.get_uptime();
            let days = uptime / 86400;
            uptime -= days * 86400;
            let hours = uptime / 3600;
            uptime -= hours * 3600;
            let minutes = uptime / 60;
            writeln!(
                &mut io::stdout(),
                "{} days {} hours {} minutes ({} seconds in total)",
                days,
                hours,
                minutes,
                up,
            );
        }
        x if x.starts_with("refresh") => {
            if x == "refresh" {
                writeln!(&mut io::stdout(), "Getting processes' information...");
                sys.refresh_all();
                writeln!(&mut io::stdout(), "Done.");
            } else if x.starts_with("refresh ") {
                writeln!(&mut io::stdout(), "Getting process' information...");
                if let Some(pid) = x
                    .split(' ')
                    .filter_map(|pid| pid.parse().ok())
                    .take(1)
                    .next()
                {
                    if sys.refresh_process(pid) {
                        writeln!(&mut io::stdout(), "Process `{}` updated successfully", pid);
                    } else {
                        writeln!(
                            &mut io::stdout(),
                            "Process `{}` couldn't be updated...",
                            pid
                        );
                    }
                } else {
                    writeln!(&mut io::stdout(), "Invalid [pid] received...");
                }
            } else {
                writeln!(
                    &mut io::stdout(),
                    "\"{}\": Unknown command. Enter 'help' if you want to get the commands' \
                     list.",
                    x
                );
            }
        }
        e => {
            writeln!(
                &mut io::stdout(),
                "\"{}\": Unknown command. Enter 'help' if you want to get the commands' \
                 list.",
                e
            );
        }
    }
    false
}

fn main() {
    println!("Getting processes' information...");
    let mut t = System::new_all();
    println!("Done.");
    let t_stin = io::stdin();
    let mut stin = t_stin.lock();
    let mut done = false;

    println!("To get the commands' list, enter 'help'.");
    while !done {
        let mut input = String::new();
        write!(&mut io::stdout(), "> ");
        io::stdout().flush();

        stin.read_line(&mut input);
        if input.is_empty() {
            // The string is empty, meaning there is no '\n', meaning
            // that the user used CTRL+D so we can just quit!
            println!("\nLeaving, bye!");
            break;
        }
        if (&input as &str).ends_with('\n') {
            input.pop();
        }
        done = interpret_input(input.as_ref(), &mut t);
    }
}

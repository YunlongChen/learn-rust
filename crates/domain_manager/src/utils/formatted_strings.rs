use std::cmp::min;
use std::net::IpAddr;

use chrono::{DateTime, Local};

/// Application version number (to be displayed in gui footer)
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn print_cli_welcome_message() {
    print!(
        r"
  /---------------------------------------------------------\
 |     _____           _    __    __                  _      |
 |    / ____|         (_)  / _|  / _|                | |     |
 |   | (___    _ __    _  | |_  | |_   _ __     ___  | |_    |
 |    \___ \  | '_ \  | | |  _| |  _| | '_ \   / _ \ | __|   |
 |    ____) | | | | | | | | |   | |   | | | | |  __/ | |_    |
 |   |_____/  |_| |_| |_| |_|   |_|   |_| |_|  \___|  \__|   |
 |                                                           |
 |                   ___________                             |
 |                  /___________\                            |
 |                 | ___________ |                           |
 |                 | |         | |                           |
 |                 | | v{APP_VERSION}  | |                   |
 |                 | |_________| |________________________   |
 |                 \_____________/   by Stan              )  |
 |                 / ''''''''''' \                       /   |
 |                / ::::::::::::: \                  =D-'    |
 |               (_________________)                         |
  \_________________________________________________________/
    "
    );
}

pub fn get_domain_from_r_dns(r_dns: String) -> String {
    if r_dns.parse::<IpAddr>().is_ok() || r_dns.is_empty() {
        // rDNS is equal to the corresponding IP address (can't be empty but checking it to be safe)
        r_dns
    } else {
        let parts: Vec<&str> = r_dns.split('.').collect();
        let len = parts.len();
        if len >= 2 {
            let last = parts.get(len - 1).unwrap_or(&"");
            let second_last = parts.get(len - 2).unwrap_or(&"");
            if last.len() > 3 || second_last.len() > 3 {
                format!("{second_last}.{last}")
            } else {
                let third_last_opt = len.checked_sub(3).and_then(|i| parts.get(i));
                match third_last_opt {
                    Some(third_last) => format!("{third_last}.{second_last}.{last}"),
                    None => format!("{second_last}.{last}"),
                }
            }
        } else {
            r_dns
        }
    }
}

pub fn get_socket_address(address: &IpAddr, port: Option<u16>) -> String {
    if let Some(res) = port {
        if address.is_ipv6() {
            // IPv6
            format!("[{address}]:{res}")
        } else {
            // IPv4
            format!("{address}:{res}")
        }
    } else {
        address.to_string()
    }
}

pub fn get_path_termination_string(full_path: &str, i: usize) -> String {
    let chars = full_path.chars().collect::<Vec<char>>();
    if chars.is_empty() {
        return String::new();
    }
    let tot_len = chars.len();
    let slice_len = min(i, tot_len);
    let suspensions = if tot_len > i { "…" } else { "" };
    [
        suspensions,
        &chars[tot_len - slice_len..].iter().collect::<String>(),
        " ",
    ]
        .concat()
}

pub fn get_formatted_num_seconds(num_seconds: u128) -> String {
    match num_seconds {
        0..3600 => format!("{:02}:{:02}", num_seconds / 60, num_seconds % 60),
        _ => format!(
            "{:02}:{:02}:{:02}",
            num_seconds / 3600,
            (num_seconds % 3600) / 60,
            num_seconds % 60
        ),
    }
}

pub fn get_formatted_timestamp(t: DateTime<Local>) -> String {
    t.format("%H:%M:%S").to_string()
}

#[allow(dead_code)]
#[cfg(windows)]
pub fn get_logs_file_path() -> Option<String> {
    let mut conf =
        confy::get_configuration_file_path(crate::DOMAIN_MANAGER_LOWERCASE, "logs").ok()?;
    conf.set_extension("txt");
    Some(conf.to_str()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatted_num_seconds() {
        assert_eq!(get_formatted_num_seconds(0), "00:00");
        assert_eq!(get_formatted_num_seconds(1), "00:01");
        assert_eq!(get_formatted_num_seconds(28), "00:28");
        assert_eq!(get_formatted_num_seconds(59), "00:59");
        assert_eq!(get_formatted_num_seconds(60), "01:00");
        assert_eq!(get_formatted_num_seconds(61), "01:01");
        assert_eq!(get_formatted_num_seconds(119), "01:59");
        assert_eq!(get_formatted_num_seconds(120), "02:00");
        assert_eq!(get_formatted_num_seconds(121), "02:01");
        assert_eq!(get_formatted_num_seconds(3500), "58:20");
        assert_eq!(get_formatted_num_seconds(3599), "59:59");
        assert_eq!(get_formatted_num_seconds(3600), "01:00:00");
        assert_eq!(get_formatted_num_seconds(3601), "01:00:01");
        assert_eq!(get_formatted_num_seconds(3661), "01:01:01");
        assert_eq!(get_formatted_num_seconds(7139), "01:58:59");
        assert_eq!(get_formatted_num_seconds(7147), "01:59:07");
        assert_eq!(get_formatted_num_seconds(7199), "01:59:59");
        assert_eq!(get_formatted_num_seconds(7200), "02:00:00");
        assert_eq!(get_formatted_num_seconds(9999), "02:46:39");
        assert_eq!(get_formatted_num_seconds(36000), "10:00:00");
        assert_eq!(get_formatted_num_seconds(36001), "10:00:01");
        assert_eq!(get_formatted_num_seconds(36061), "10:01:01");
        assert_eq!(get_formatted_num_seconds(86400), "24:00:00");
        assert_eq!(get_formatted_num_seconds(123456789), "34293:33:09");
        assert_eq!(
            get_formatted_num_seconds(u128::MAX),
            "94522879700260684295381835397713392:04:15"
        );
    }

    #[test]
    fn test_formatted_timestamp() {
        let now = Local::now();
        let formatted = get_formatted_timestamp(now);
        let expected = now.to_string().get(11..19).unwrap().to_string();
        assert_eq!(formatted, expected);
    }

    #[cfg(windows)]
    #[test]
    fn test_logs_file_path() {
        let file_path = std::path::PathBuf::from(get_logs_file_path().unwrap());
        assert!(file_path.is_absolute());
        assert_eq!(file_path.file_name().unwrap(), "logs.txt");
    }

    #[test]
    fn test_get_domain_from_r_dns() {
        let f = |s: &str| get_domain_from_r_dns(s.to_string());
        assert_eq!(f(""), "");
        assert_eq!(f("8.8.8.8"), "8.8.8.8");
        assert_eq!(f("a.b.c.d"), "b.c.d");
        assert_eq!(f("ciao.xyz"), "ciao.xyz");
        assert_eq!(f("bye.ciao.xyz"), "ciao.xyz");
        assert_eq!(f("ciao.bye.xyz"), "ciao.bye.xyz");
        assert_eq!(f("hola.ciao.bye.xyz"), "ciao.bye.xyz");
        assert_eq!(f(".bye.xyz"), ".bye.xyz");
        assert_eq!(f("bye.xyz"), "bye.xyz");
        assert_eq!(f("hola.ciao.b"), "ciao.b");
        assert_eq!(f("hola.b.ciao"), "b.ciao");
        assert_eq!(f("ciao."), "ciao.");
        assert_eq!(f("ciao.."), "ciao..");
        assert_eq!(f(".ciao."), "ciao.");
        assert_eq!(f("ciao.bye."), "ciao.bye.");
        assert_eq!(f("ciao..."), "..");
        assert_eq!(f("..bye"), "..bye");
        assert_eq!(f("ciao..bye"), "ciao..bye");
        assert_eq!(f("..ciao"), ".ciao");
        assert_eq!(f("bye..ciao"), ".ciao");
        assert_eq!(f("."), ".");
        assert_eq!(f(".."), "..");
        assert_eq!(f("..."), "..");
        assert_eq!(f("no_dots_in_this"), "no_dots_in_this");
    }
}

use regex::Regex;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let regex = Regex::new(r"(?m)^(?P<timestamp>\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(?P<host>[\w-]+(?:\s+[\w-]+)*)\s+(?P<process>[\w-]+\[\d+\])?:\s+(?P<message>.+)$").unwrap();
        let string = "Nov  3 16:36:42 depot-MacBook-Air login[2735]: DEAD_PROCESS: 2735 ttys003
        
        ";
        
        if let Some(caps) = regex.captures(string) {
            let timestamp = caps.name("timestamp").map(|m| m.as_str()).unwrap_or_default();
            let host = caps.name("host").map(|m| m.as_str()).unwrap_or_default();
            let process = caps.name("process").map(|m| m.as_str()).unwrap_or_default();
            let message = caps.name("message").map(|m| m.as_str()).unwrap_or_default();

            assert_eq!(timestamp, "Nov  3 16:36:42");
            assert_eq!(host, "depot-MacBook-Air");
            assert_eq!(process, "login[2735]");
            assert_eq!(message, "DEAD_PROCESS: 2735 ttys003");
        } else {
            println!("No matches found.");
            assert!(false, "Expected captures but found none.");
        }
    }
}

// etherhosts: create hosts and ethers files from CSV
// By David Atkinson 2021
// CSV parsing is primitive, but should handle quoted strings

use std::env;
use std::fs; 
use chrono::{DateTime, Local};
use regex::Regex;

fn process_csv_line(txt: &str) -> Vec<String> {                  
    // This function splits a CSV line at commas, and handles basic quoted text
    // It does not handle multiline text
    
    // Make a copy of the input string
    let mut line = String::from(txt);
    
    // A "" should be translated to a single quote
    // Replace "" with space, and store the location
    let mut ddq = Vec::<usize>::new();   
    let mut f = line.find("\"\"");
    while f.is_some() {
        let i = f.unwrap();
        ddq.push(i);
        line.replace_range(i..=i+1, " ");
        f = line.find("\"\"");
    }

    // Now there is only single (bounding) quotes and commas, it's easier to parse
    // e.g. cell1,"cell2, with comma",cell3    
    let mut quoted: bool = false;
    let mut cells = Vec::<String>::new();
    let mut cell = String::new();
    
    for (i, b) in line.into_bytes().iter().enumerate() {
        let c = *b as char;
        if c == '"' {
            quoted = !quoted;
        } else if !quoted && c == ',' {
            cells.push(cell.clone());
            cell.clear();
        } else if ddq.contains(&i) {    // put the " back in
            cell.push('"');
        } else {
            cell.push(c);
        }
    }

    cells.push(cell);
    
    return cells;
}

fn clean_ipaddr(s: &str) -> Result<String, String>  {
	// This function checks (and performs minor cleaning of) an ipv4 dotted decimal address.
	// It returns an Ok(String) for a valid ipaddr, and
	// Err(String) for an erroneous (or missing) address.
	
	// trim both sides of any extra whitespace
	let s = s.trim();
	
	// check it matches a basic ipv4 dotted decimal pattern
	let re = Regex::new(r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)(\.)){3}(25[0-5]|(2[0-4]|1\d|[1-9]|)\d)$").unwrap();	

	// return a copy of the ipaddr if it matches, otherwise return an error
	if re.is_match(s) {
		return Ok(s.to_string());
	} else {
		return Err("ipaddr failed regex check".to_string());
	}
}

fn clean_mac(s: &str) -> Result<String, String> {
	// This function checks (and performs minor cleaning of) a mac address.
	// It returns an Ok(String) for a valid mac address, and
	// Err("") for a blank macaddr, and 
	// Err(String) for an erroneous address.

	// trim both sides of any extra whitespace
	let s = s.trim();
	
	// if the macaddr is empty, return a blank string as err
	if s.len() == 0 {
	    return Err("".to_string());
	}

	// convert windows hyphens into colons
	let s: String = s.chars().map(|c| 
		if c=='-' { 
			return ':';
		} else {
			return c;
		}
	).collect();
	
	// make the macaddr lowercase
	let s: String = s.to_lowercase();
	
	// check the string matches a basic mac address pattern
	let re = Regex::new(r"^[a-f0-9]{2}(:[a-f0-9]{2}){5}$").unwrap();

	// return the mac address if it matches, otherwise return an error
	if re.is_match(&s) {
		return Ok(s);
	} else {
		return Err("macaddr failed regex check".to_string());
	}
}

fn clean_hostname(s: &str) -> Result<String, String> {
	// This function checks (and performs minor cleaning of) a hostname.
	// It returns an Ok(String) for a valid hostname, and
	// Err("") for a blank hostname, and 
	// Err(String) for an erroneous hostname.

	// trim both sides of any extra whitespace
	let s: String = s.trim().to_string();

	// if the hostname is empty, return a blank string as err
	if s.len() == 0 {
	    return Err("".to_string());
	}

	// check for obvious mistakes in hostname
	let re = Regex::new(r"^([a-zA-Z0-9-\. ]*)$").unwrap();

	if re.is_match(&s) {
		return Ok(s);
	} else {
		return Err("hostname failed regex check".to_string());
	}
}

fn main() {
    // display program info
    println!("Etherhosts: Create hosts and ethers files from CSV");
    println!("Usage: etherhosts [etherhosts.csv] [hosts] [ethers]");

    // filenames
    let mut inputfile = "etherhosts.csv";
    let mut hostsfile = "hosts";
    let mut ethersfile = "ethers";

    // get filenames from command line
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        inputfile = &args[1];
    }
    if args.len() >= 3 {
        hostsfile = &args[2];
    }
    if args.len() >= 4 {
        ethersfile = &args[3];
    }

    // read input file
    let input = fs::read_to_string(inputfile).expect("Unable to open input file");
    let mut lines = input.lines();

    // read first line to determine positions of each column
    let header_row = process_csv_line(&lines.next().expect("Input file didn't have a single line!").to_string());
    let mut ipaddrcol: usize = 0;
    let mut hostnamecol: usize = 0;
    let mut maccol: usize = 0;
    let mut found_i = false;
    let mut found_h = false;
    let mut found_m = false;

    for (c, field) in header_row.iter().enumerate() {
        if field == "ipaddr" {
            found_i = true;
            ipaddrcol = c;
        } else if field == "hostname" {
            found_h = true;
            hostnamecol = c;
        } else if field == "macaddr" {
            found_m = true;
            maccol = c;
        }
    }

    if !(found_i && found_h && found_m) {
        println!("Couldn't find all the headers: ipaddr, hostname, macaddr");
        return;
    }

    // display our input and output file names
    println!("Input csv:     {}\nOutput hosts:  {}\nOutput ethers: {}", inputfile, hostsfile, ethersfile);

    // hosts and ethers to be stored in strings
    let mut hoststxt = String::new();
    let mut etherstxt = String::new();

    // header for hosts and ethers files
    let now: DateTime<Local> = Local::now();
    let timestr = now.format("%F %T %Z");

    hoststxt.push_str(&format!("# hosts automatically generated by etherhosts {}\n", &timestr));
    etherstxt.push_str(&format!("# ethers automatically generated by etherhosts {}\n", &timestr));

    // process each line of the input file
    for (r,line) in lines.enumerate() {
        let fields = process_csv_line(&line.to_string());

	let ipaddr = match clean_ipaddr(&fields[ipaddrcol]) {
	    Ok(s)  => s,
	    Err(s) => {
		println!("skipping line {}: {}", r+2, s);
		continue;
	    }
	};
		
	match clean_hostname(&fields[hostnamecol]) {
	    Ok(hostname)  => {
		// add to hosts
		// ipaddr can be padded using {: <15}
		let hostline = format!("{} {}\n", ipaddr, hostname);
		hoststxt.push_str(&hostline);
	    },
	    Err(s) => {
		// If the string is empty, it's simply a blank hostname and not a real error
		if s.len() != 0 {
		    println!("invalid hostname on line {}: {}", r+2, s);
		}
	    }
	}
	    
	match clean_mac(&fields[maccol]) {
	    Ok(macaddr)  => {
		// add to ethers
		let etherline = format!("{} {}\n", macaddr, ipaddr);
		etherstxt.push_str(&etherline);
	    },
	    Err(s) => {
		// If the string is empty, it's simply a blank macaddr and not a real error
		if s.len() != 0 {
		    println!("invalid macaddr on line {}: {}", r+2, s);
		}
	    }
        }
    }

    // write to output files
    if fs::write(hostsfile, hoststxt).is_err() {
        println!("Unable to write to hosts file");
    }
    if fs::write(ethersfile, etherstxt).is_err() {
        println!("Unable to write to ethers file");
    }

}

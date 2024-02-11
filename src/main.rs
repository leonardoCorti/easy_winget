use std::process::{Command, exit};

#[derive(Debug)]
struct Program{
    name: String,
    id: String,
    is_id_complete: bool,
}

impl Program{

    fn get_identifier(&self) -> &str {
        match self.is_id_complete{
            true => &self.id,
            false => &self.name,
        }
    }


    fn update(&self) -> Result<(),std::io::Error>{
        let identifier = match self.is_id_complete{
            true => &self.id,
            false => &self.name,
        };

        let output = Command::new("cmd")
            .args(&["/C", "winget", "upgrade", "-h", "-s", "winget" , identifier ])
            .output();
        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    Ok(())
                } else {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    eprintln!("Command failed with exit code: {:?}", output.status);
                    //exit(output.status.code().unwrap_or(1));
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Some error message"));
                }
            }
            Err(err) => Err(err),
        }
    }
}

impl std::str::FromStr for Program {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //at 32 the name finishes and the id starts

        let name : String = s.chars().take(34).collect();
        let id: String = s.chars().rev().take(35).collect::<String>().chars().rev().collect();
        let is_id_complete = !id.contains('…');
        let name = &name.replace('…', "");

        return Ok(Program{ name: name.trim().to_string(), id: id.trim().to_string(), is_id_complete });
    }
}

fn main() -> Result<(), std::io::Error> {
    print!("reading packages ...");
    let output = run_command("winget upgrade")?;
    let from_utf8_lossy = String::from_utf8_lossy(&output.stdout);
    let programs_strings: Vec<String> = from_utf8_lossy.lines()
        .filter(|lin| lin.contains("winget"))
        .map(|lin|
        lin.chars().take(69).collect::<String>()
    ).collect();

    let programs: Vec<Program> = programs_strings.iter().map(|pr| pr.parse().unwrap() ).collect(); 

    println!("List of programs ready for an update:");
    for (index, program) in programs.iter().enumerate(){
        println!("{:>2}  {}", index, program.get_identifier());
    }

    let mut correct_input = false;
    let mut selected_programs: Vec<usize>= Vec::new();
    while !correct_input {
        correct_input = true;
        println!("Choose the programs to update, select numbers like 1,2,3,5,6 or ranges like 1-3,5-6");
        let mut user_input = String::new();
        let _ = std::io::stdin().read_line(&mut user_input)?;
        match elaborate_input(&user_input.trim()){
            Ok(result) => selected_programs = result,
            Err(_) => correct_input=false,
        }
    }

    println!("Updating...");

    for (index, program) in programs.iter().enumerate(){
        if selected_programs.contains(&index){
            println!("Updating {:#?}", &program);
            match program.update() {
                Ok(_) => println!("{} updated", program.name),
                Err(_) => println!("{} NOT updated", program.name),
            }
        }
    }


    Ok(())
}

enum InputParseError{
    BadFormat,
}

fn elaborate_input(user_input: &str) -> Result<Vec<usize>,InputParseError> {
    let mut selected_numbers: Vec<usize> = Vec::new();

    if let Ok(number) = user_input.parse::<usize>(){
        //if the input is a single number
        selected_numbers.push(number);
    } else if user_input.contains('-') && !user_input.contains(','){
        //if the input is just a range
        match extract_first_and_last(&user_input){
            Ok((first, last)) => {
                for i in first..=last{
                    selected_numbers.push(i);
                }
            },
            Err(_) => return Err(InputParseError::BadFormat),
        }
    } else if !user_input.contains(','){
        return Err(InputParseError::BadFormat);
    } else {
        //it is a list of ranges and single numbers at this point
        let selections: Vec<&str> = user_input.split(',').collect();
        for a_selection in selections{
            if a_selection.contains('-'){
                match extract_first_and_last(&a_selection){
                    Ok((first, last)) => {
                        for i in first..=last{
                            selected_numbers.push(i);
                        }
                    },
                    Err(_) => return Err(InputParseError::BadFormat),
                }
            } else {
                if let Ok(number) = a_selection.parse::<usize>(){
                    selected_numbers.push(number);
                } else {
                    return Err(InputParseError::BadFormat);
                }
            }
        }
    }

    return Ok(selected_numbers);
}

fn extract_first_and_last(user_input: &str) -> Result<(usize,usize),InputParseError> {
    let numbers: Vec<usize> = user_input.split('-').map(|n| n.parse::<usize>().unwrap_or(0)).collect();
    if numbers.iter().all(|n| *n==0) {
        return Err(InputParseError::BadFormat);
    }
    return Ok((numbers[0],numbers[1]));
}

fn run_command(command: &str) -> Result<std::process::Output, std::io::Error> {
    let output = Command::new("cmd")
        .args(&["/C", command])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(output)
            } else {
                eprintln!("Command failed with exit code: {:?}", output.status);
                exit(output.status.code().unwrap_or(1));
            }
        }
        Err(err) => Err(err),
    }
}

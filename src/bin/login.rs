use anyhow::Result;
use iuc_bot::client::aksis::AksisClient;
use iuc_bot::client::obs::ObsClient;
use std::io;
use std::io::Write;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: cargo run <year> <term(1,2)>");
        return Ok(());
    }

    let year = &args[1];
    let term = &args[2];

    let mut aksis = AksisClient::new();

    let username = aksis
        .cache
        .username
        .clone()
        .or_else(|| {
            print!("Enter username: ");
            io::stdout().flush().unwrap();
            let mut u = String::new();
            io::stdin().read_line(&mut u).unwrap();
            Some(u.trim().to_string())
        })
        .unwrap();
    let password = aksis
        .cache
        .password
        .clone()
        .or_else(|| {
            print!("Enter password: ");
            io::stdout().flush().unwrap();
            let mut p = String::new();
            io::stdin().read_line(&mut p).unwrap();
            Some(p.trim().to_string())
        })
        .unwrap();

    let auth_cookie = match aksis.login(&username, &password) {
        Ok((Some(cookie), _)) => cookie,
        Ok((None, csrf_token)) => {
            print!("Enter SMS code: ");
            io::stdout().flush()?;
            let mut code = String::new();
            io::stdin().read_line(&mut code)?;
            aksis.send_sms(code.trim(), &csrf_token)?
        }
        Err(e) => {
            println!("{e}");
            return Ok(());
        }
    };

    println!("{}", auth_cookie);

    let obs = ObsClient::new(auth_cookie);
    let results = obs.get_exam_results(year, term)?;

    println!("{:#?}", results);

    Ok(())
}

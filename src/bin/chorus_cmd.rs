use chorus::error::{ChorusError, Error};
use pocket_types::{Filter, Id, Pubkey, Tags};
use std::env;

const USAGE: &str = "usage: chorus_cmd <config_path> <command> [args...]";

fn main() -> Result<(), Error> {
    // Get args (config path)
    let mut args = env::args();
    if args.len() <= 1 {
        panic!("USAGE: chorus_moderate <config_path>");
    }
    let _ = args.next(); // ignore program name

    // Load config
    let config_path = args
        .next()
        .ok_or::<Error>(ChorusError::General(USAGE.to_owned()).into())?;
    let mut config = chorus::load_config(config_path)?;
    // Force allow of scraping (this program is a scraper)
    config.allow_scraping = true;

    // Setup store
    let store = chorus::setup_store(&config)?;

    // Setup logging
    chorus::setup_logging(&config);

    // Handle command
    let command = args
        .next()
        .ok_or::<Error>(ChorusError::General(USAGE.to_owned()).into())?;
    match &*command {
        "delete_by_id" => {
            let idstr = args
                .next()
                .ok_or::<Error>(ChorusError::General("ID argument missing".to_owned()).into())?;
            let id: Id = Id::read_hex(idstr.as_bytes())?;
            store.remove_event(id)?;
            println!("Done.");
        }
        "delete_by_pubkey" => {
            let pubstr = args.next().ok_or::<Error>(
                ChorusError::General("Pubkey argument missing".to_owned()).into(),
            )?;
            let pk: Pubkey = Pubkey::read_hex(pubstr.as_bytes())?;

            let mut tags_buffer: [u8; 128] = [0; 128];
            let (_, tags) = Tags::from_json(b"[]", &mut tags_buffer)?;
            let mut filter_buffer: [u8; 128] = [0; 128];
            let filter =
                Filter::from_parts(&[], &[pk], &[], tags, None, None, None, &mut filter_buffer)?;
            let events = store.find_events(filter, true, 0, 0, |_| true)?;
            for event in events.iter() {
                store.remove_event(event.id())?;
            }
            println!("Done.");
        }
        _ => {
            return Err(ChorusError::General("Unknown command.".to_owned()).into());
        }
    }

    Ok(())
}

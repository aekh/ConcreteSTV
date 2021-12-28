// Copyright 2021 Andrew Conway.
// This file is part of ConcreteSTV.
// ConcreteSTV is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// ConcreteSTV is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


use std::fs::File;
use nsw::NSWECLocalGov2021;
use nsw::parse_lge::{get_nsw_lge_data_loader_2021, NSWLGEDataLoader};
use stv::ballot_metadata::CandidateIndex;
use stv::distribution_of_preferences_transcript::TranscriptWithMetadata;
use stv::parse_util::{FileFinder, RawDataSource};
use stv::preference_distribution::{distribute_preferences, PreferenceDistributionRules};
use stv::tie_resolution::TieResolutionsMadeByEC;

mod test_nsw_lge;


fn test<Rules:PreferenceDistributionRules,F:Fn(Rules::Tally)->f64>(electorate:&str,loader:&NSWLGEDataLoader,decode:F) {
    let data = loader.load_cached_data(electorate).unwrap();
    data.print_summary();
    let mut tie_resolutions = TieResolutionsMadeByEC::default();
    let official_transcript = loader.read_official_dop_transcript(&data.metadata).unwrap();
    loop {
        let transcript = distribute_preferences::<Rules>(&data, loader.candidates_to_be_elected(electorate), &data.metadata.excluded.iter().cloned().collect(), &tie_resolutions);
        let transcript = TranscriptWithMetadata{ metadata: data.metadata.clone(), transcript };
        std::fs::create_dir_all("test_transcripts").unwrap();
        {
            let file = File::create(format!("test_transcripts/NSW LG{} {}.transcript",transcript.metadata.name.year,electorate)).unwrap();
            serde_json::to_writer_pretty(file,&transcript).unwrap();
        }
        if let Some((favoured_candidate,unfavoured_candidate)) = official_transcript.compare_with_transcript_checking_for_ec_decisions(&transcript.transcript,&decode) {
            println!("Adding tie resolution {}>{}",favoured_candidate,unfavoured_candidate);
            assert!(favoured_candidate.0<unfavoured_candidate.0,"favoured candidate should be lower as higher candidates are assumed favoured.");
            if tie_resolutions.tie_resolutions.contains(&vec![unfavoured_candidate,favoured_candidate]) {
                panic!("That tie resolution is already in the list.")
            }
            tie_resolutions.tie_resolutions.push(vec![unfavoured_candidate,favoured_candidate]);
        } else {
            return;
        }
    }
}

fn decode(tally:usize) -> f64 { tally as f64 }

#[test]
fn test_ineligible() {
    let finder = FileFinder::find_ec_data_repository();
    println!("Found files at {:?}",finder.path);
    let loader = get_nsw_lge_data_loader_2021(&finder).unwrap();
    let data = loader.read_raw_data_checking_electorate_valid("Ballina - B Ward").unwrap();
    assert_eq!(data.metadata.excluded,vec![CandidateIndex(2)]);
}
#[test]
fn test_all_council_races() {
    let finder = FileFinder::find_ec_data_repository();
    println!("Found files at {:?}",finder.path);
    let loader = get_nsw_lge_data_loader_2021(&finder).unwrap();
    println!("Made loader");
    let electorate =&loader.all_electorates()[0];
    assert_eq!(electorate,"City of Albury");
    for electorate in &loader.all_electorates() {
        if !electorate.ends_with(" Mayoral") {
            println!("Testing Electorate {}",electorate);
            test::<NSWECLocalGov2021,_>(electorate,&loader,decode);
        }
    }
}

/*
#[test]
fn make_stv_file_of_everything() {
    let finder = FileFinder::find_ec_data_repository();
    println!("Found files at {:?}",finder.path);
    let loader = get_nsw_lge_data_loader_2021(&finder).unwrap();
    println!("Made loader");
    std::fs::create_dir_all("test_stv_files").unwrap();
    for electorate in loader.all_electorates() {
        println!("Trying to load {}",&electorate);
        let data = loader.read_raw_data(&electorate).unwrap();
        data.print_summary();
        let file = File::create(format!("test_stv_files/NSW LG{} {}.stv",data.metadata.name.year,electorate)).unwrap();
        serde_json::to_writer(file,&data).unwrap();
    }
}*/

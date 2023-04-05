// Copyright 2021-2022 Andrew Conway.
// This file is part of ConcreteSTV.
// ConcreteSTV is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// ConcreteSTV is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


use nsw::parse_lge::get_nsw_lge_data_loader_2017;
use stv::parse_util::{FileFinder, RawDataSource};


/*
fn test<Rules:PreferenceDistributionRules>(electorate:&str,loader:&NSWLGEDataLoader) {
    let data = loader.read_raw_data(electorate).unwrap();
    data.print_summary();
    let mut tie_resolutions = TieResolutionsMadeByEC::default();
    let official_transcript = loader.read_official_dop_transcript(&data.metadata).unwrap();
    loop {
        let transcript = distribute_preferences::<Rules>(&data, loader.candidates_to_be_elected(electorate), &data.metadata.excluded.iter().cloned().collect(), &tie_resolutions,None,false);
        let transcript = TranscriptWithMetadata{ metadata: data.metadata.clone(), transcript };
        std::fs::create_dir_all("test_transcripts").unwrap();
        {
            let file = File::create(format!("test_transcripts/NSW LG{} {}.transcript",transcript.metadata.name.year,electorate)).unwrap();
            serde_json::to_writer_pretty(file,&transcript).unwrap();
        }
        if let Some(decision) = official_transcript.compare_with_transcript_checking_for_ec_decisions(&transcript.transcript,true).unwrap() {
            println!("Observed tie resolution favouring {:?} over {:?}", decision.favoured, decision.disfavoured);
            assert!(decision.favoured.iter().map(|c|c.0).min().unwrap() < decision.disfavoured[0].0, "favoured candidate should be lower as higher candidates are assumed favoured.");
            tie_resolutions.tie_resolutions.push(TieResolutionAtom::ExplicitDecision(decision));
        } else {
            return;
        }
    }
}

#[test]
fn test_ineligible() {
    let finder = FileFinder::find_ec_data_repository();
    println!("Found files at {:?}",finder.path);
    let loader = get_nsw_lge_data_loader_2021(&finder).unwrap();
    let data = loader.read_raw_data_checking_electorate_valid("Ballina - B Ward").unwrap();
    assert_eq!(data.metadata.excluded,vec![CandidateIndex(2)]);
}
*/


#[test]
fn test_2017_loadable() {
    let finder = FileFinder::find_ec_data_repository();
    println!("Found files at {:?}",finder.path);
    let loader = get_nsw_lge_data_loader_2017(&finder).unwrap();
    println!("Made loader");
    let electorate =&loader.all_electorates()[0];
    assert_eq!(electorate,"Armidale Regional");
    for electorate in &loader.all_electorates() {
        println!("Testing Electorate {}",electorate);
//        test::<NSWECLocalGov2021>(electorate,&loader);
        let metadata = loader.read_raw_metadata(electorate).unwrap();
        println!("{:?}",metadata);
        let data = loader.read_raw_data(electorate).unwrap();
        data.print_summary();
        let _dop = loader.read_official_dop_transcript(&metadata).unwrap();
    }
}

/*

From a prior project we have estimates of probability of different candidates winning:

North Sydney Wollstonecraft Ward
Candidate	Proportion Elected	Mean position	Official Count
BAKER Zoe	1.000000	1.000000	1
MUTTON Ian	1.000000	2.000000	2
GUNNING Samuel	0.789956	3.000000	3
KELLY Tim	0.210044	3.000000

 */
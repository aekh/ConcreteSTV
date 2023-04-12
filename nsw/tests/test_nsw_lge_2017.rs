// Copyright 2021-2022 Andrew Conway.
// This file is part of ConcreteSTV.
// ConcreteSTV is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// ConcreteSTV is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.


use std::fs::File;
use nsw::nsw_random_rules::{NSWECrandomLGE2017};
use nsw::parse_lge::{get_nsw_lge_data_loader_2017, NSWLGEDataLoader, NSWLGEDataSource};
use stv::distribution_of_preferences_transcript::TranscriptWithMetadata;
use stv::official_dop_transcript::{DifferenceBetweenOfficialDoPAndComputed, test_official_dop_without_actual_votes};
use stv::parse_util::{FileFinder, RawDataSource};
use stv::preference_distribution::{distribute_preferences, PreferenceDistributionRules};
use stv::tie_resolution::{TieResolutionAtom, TieResolutionExplicitDecision};


fn test<Rules:PreferenceDistributionRules>(electorate:&str,loader:&NSWLGEDataLoader) {
    let data = loader.read_raw_data(electorate).unwrap();
    data.print_summary();
    let mut tie_resolutions = data.metadata.tie_resolutions.clone();
    /*
    if electorate=="Port Stephens - Central Ward" {
        tie_resolutions.tie_resolutions.push(TieResolutionAtom::ExplicitDecision(TieResolutionExplicitDecision{
            favoured: vec![CandidateIndex(1)],
            disfavoured: vec![CandidateIndex(2),CandidateIndex(13),CandidateIndex(14),CandidateIndex(15)],
            came_up_in: Some("2".to_string()),
        }))
    }*/
    let official_transcript = loader.read_official_dop_transcript(&data.metadata).unwrap();
    loop {
        let transcript = distribute_preferences::<Rules>(&data, loader.candidates_to_be_elected(electorate), &data.metadata.excluded.iter().cloned().collect(), &tie_resolutions,None,false);
        let transcript = TranscriptWithMetadata{ metadata: data.metadata.clone(), transcript };
        std::fs::create_dir_all("test_transcripts").unwrap();
        {
            let file = File::create(format!("test_transcripts/NSW LG{} {}.transcript",transcript.metadata.name.year,electorate)).unwrap();
            serde_json::to_writer_pretty(file,&transcript).unwrap();
        }
        match official_transcript.compare_with_transcript_checking_for_ec_decisions(&transcript.transcript,true) {
            Ok(None) => { return; }
            Ok(Some(decision)) => {
                println!("Observed tie resolution favouring {:?} over {:?}", decision.favoured, decision.disfavoured);
                assert!(decision.favoured.iter().map(|c|c.0).min().unwrap() < decision.disfavoured[0].0, "favoured candidate should be lower as higher candidates are assumed favoured.");
                tie_resolutions.tie_resolutions.push(TieResolutionAtom::ExplicitDecision(decision));
            }
            Err(DifferenceBetweenOfficialDoPAndComputed::DifferentNumbersOfCounts(official,our)) => {
                println!("Official DoP had {} counts; ConcreteSTV had {}. Not surprising as the algorithm contains random elements.",official,our);
                return;
            }
            Err(DifferenceBetweenOfficialDoPAndComputed::DifferentOnCount(count_index,_,diff)) => {
                println!("Tie resolutions : {:?}",tie_resolutions);
                println!("There was a difference between the official DoP and ConcreteSTV's on count {} : {}",1+count_index.0,diff);
                if count_index.0<2 {
                    panic!("A count error on count {} is not explainable by the random part of the algorithm : {}",1+count_index.0,diff);
                } else {
                    println!("This is probably due to the random elements of the algorithm.");
                    return;
                }
            }
            Err(e) => {
                println!("Tie resolutions : {:?}",tie_resolutions);
                panic!("There was a difference between the official DoP and ConcreteSTV's : {}",e);
            }
        }
    }
}



#[test]
fn test_2017_plausible() {
    let finder = FileFinder::find_ec_data_repository();
    println!("Found files at {:?}",finder.path);
    let loader = get_nsw_lge_data_loader_2017(&finder).unwrap();
    println!("Made loader");
    assert_eq!(&loader.all_electorates()[0],"Armidale Regional");
    for electorate in &loader.all_electorates() {
        test::<NSWECrandomLGE2017>(electorate,&loader);
        println!("Testing Electorate {}",electorate);
    }
}

#[test]
fn test_wollstonecraft() {
    let finder = FileFinder::find_ec_data_repository();
    let loader = get_nsw_lge_data_loader_2017(&finder).unwrap();
    test::<NSWECrandomLGE2017>("North Sydney - Wollstonecraft Ward",&loader);
}


#[test]
/// From a prior project we have estimates of probability of different candidates winning for North Sydney Wollstonecraft Ward:
/// ```text
/// Candidate	Proportion Elected	Mean position	Official Count
/// BAKER Zoe	1.000000	1.000000	1
/// MUTTON Ian	1.000000	2.000000	2
/// GUNNING Samuel	0.789956	3.000000	3
/// KELLY Tim	0.210044	3.000000
/// ```
///
/// Note that there is a chance that this will fail if we are absurdly unlucky.
fn test_wollstonecraft_run_1000_times_and_check_probabilistic_winners_reasonably_close_to_expected() {
    let finder = FileFinder::find_ec_data_repository();
    let loader = get_nsw_lge_data_loader_2017(&finder).unwrap();
    let data = loader.read_raw_data("North Sydney - Wollstonecraft Ward").unwrap();
    let mut num_times_elected = vec![0;data.metadata.candidates.len()];
    for _ in 0..1000 {
        let result = data.distribute_preferences::<NSWECrandomLGE2017>();
        for e in result.elected { num_times_elected[e.0]+=1; }
    }
    assert_eq!(1000,num_times_elected[3]);
    assert_eq!(1000,num_times_elected[9]);
    assert_eq!(1000,num_times_elected[0]+num_times_elected[6]);
    assert!(100<num_times_elected[6]);
    assert!(350>num_times_elected[6]);
    for candidate_index in 0..num_times_elected.len() {
        if num_times_elected[candidate_index]>0 {
            println!("Candidate {} : {} elected {} times ",candidate_index,data.metadata.candidates[candidate_index].name,num_times_elected[candidate_index]);
        }
    }
}



#[test]
fn test_2017_internally_consistent() {
    let finder = FileFinder::find_ec_data_repository();
    let loader = get_nsw_lge_data_loader_2017(&finder).unwrap();
    for electorate in &loader.all_electorates() {
        // there is something bizarre in the Federation DoP. On the NSWEC website, Federation, count 35, the second candidate WALES Norm ended the count with 623 votes. But on count 36, Wales Norm started the count with 630 votes. Other people also magically change tally. There seems no plausible way to emulate this.
        // there is something bizarre in the Inner West - Marrickville Ward DoP. On the NSWEC website, count 12, the webpage is not a count webpage but rather a duplicate of the DoP summary page.
        if electorate!="Federation" && electorate!="Inner West - Marrickville Ward" {
            println!("Testing electorate {}",electorate);
            assert_eq!(test_internally_consistent::<NSWECrandomLGE2017>("2017",electorate).unwrap(),Ok(None));
        }
    }
}

/// Test a particular year & electorate against a particular set of rules.
/// Outermost error is IO type errors.
/// Innermost error is discrepancies with the official DoP.
fn test_internally_consistent<Rules:PreferenceDistributionRules>(year:&str,state:&str) -> anyhow::Result<Result<Option<TieResolutionExplicitDecision>, DifferenceBetweenOfficialDoPAndComputed<Rules::Tally>>> where <Rules as PreferenceDistributionRules>::Tally: Send+Sync+'static {
    test_official_dop_without_actual_votes::<Rules,_>(&NSWLGEDataSource{},year,state,false)
}


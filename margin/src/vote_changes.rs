// Copyright 2022 Andrew Conway.
// This file is part of ConcreteSTV.
// ConcreteSTV is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// ConcreteSTV is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.

//! Describe a possible change in votes that may change the outcome of the election.
//! An estimate of the margin is the smallest such change that one could find.


use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{AddAssign, Sub, SubAssign};
use std::str::FromStr;
use num_traits::Zero;
use stv::ballot_metadata::CandidateIndex;
use serde::Serialize;
use serde::Deserialize;
use stv::ballot_paper::{ATL, BTL};
use stv::ballot_pile::BallotPaperCount;
use stv::election_data::ElectionData;
use stv::preference_distribution::{PreferenceDistributionRules, RoundUpToUsize};
use stv::transfer_value::TransferValue;
use crate::choose_votes::{BallotsWithGivenTransferValue, ChooseVotes, ChooseVotesOptions, TakeVotes};
use crate::retroscope::{Retroscope};

/// A list of vote changes that may change the outcome of the election
/// These are conceptual, measured in votes. There may be a larger number of ballot papers involved.
/// They can be turned into concrete actual ballots by calling [`Self::make_concrete()`].
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct VoteChanges<Tally> {
    pub changes : Vec<VoteChange<Tally>>,
}


impl <Tally:Clone+RoundUpToUsize> VoteChanges<Tally> {
    /// Add a command to transfer n votes from candidate `from` to candidate `to`.
    pub fn transfer(&mut self, n: Tally, from: CandidateIndex, to: CandidateIndex) {
        self.changes.push(VoteChange {
            n: n.clone(),
            from: Some(from),
            to: Some(to),
        })
    }
    /// Add a command to add n votes to candidate `to`.
    pub fn add(&mut self, n: Tally, to: CandidateIndex) {
        self.changes.push(VoteChange {
            n: n.clone(),
            from: None,
            to: Some(to),
        })
    }
    /// Add a command to remove n votes from candidate `from`.
    pub fn remove(&mut self, n: Tally, from: CandidateIndex) {
        self.changes.push(VoteChange {
            n: n.clone(),
            from: Some(from),
            to: None,
        })
    }
}

impl <Tally:Clone+AddAssign+SubAssign+From<usize>+Display+PartialEq+Serialize+FromStr+Ord+Sub<Output=Tally>+Zero+Hash+Sum<Tally>+RoundUpToUsize> VoteChanges<Tally> {
    pub fn make_concrete<R:PreferenceDistributionRules<Tally=Tally>>(&self,retroscope:&Retroscope,election_data:&ElectionData,options:ChooseVotesOptions) -> Option<BallotChanges<Tally>> {
        let mut builder = BallotChangesBuilder{ map: HashMap::new() };
        let mut choosers : HashMap<CandidateIndex,ChooseVotes> = HashMap::new();
        let (atl_ok_changes,btl_only_changes):(Vec<_>,Vec<_>) = self.changes.iter().partition(|vc|vc.to.map(|c|retroscope.is_highest_continuing_member_party_ticket(c,&election_data.metadata)).unwrap_or(true));
        for (change,allow_atl) in btl_only_changes.iter().map(|&x|(x,false)).chain(atl_ok_changes.iter().map(|&x|(x,true))) {
            if let Some(from) = change.from {
                let chooser = choosers.entry(from).or_insert_with(||retroscope.get_chooser(from,election_data,options));
                if let Some(ballots) = chooser.get_votes::<R>(change.n.clone(),allow_atl) {
                    for b in ballots {
                        builder.add(change.from,change.to,b);
                    }
                } else { return None; } // could not find the requisite votes.
            } else {
                if let Some(_to) = change.to { // insert votes
                    builder.add(change.from,change.to,BallotsWithGivenTransferValue{
                        n: BallotPaperCount(change.n.ceil()),
                        tally: change.n.clone(),
                        tv: TransferValue::one(),
                        ballots: vec![],
                    });
                } else { eprintln!("Trying to do a vote change that does nothing."); } // don't actually do anything...
            }

        }
        Some(builder.to_ballot_changes())
    }
}

#[derive(Clone,Debug,Eq,PartialEq,Hash)]
struct BallotChangesKey {
    from : Option<(TransferValue,CandidateIndex)>,
    to : Option<CandidateIndex>,
}
/// Utility to build a BallotChanges object.
struct BallotChangesBuilder<Tally> {
    map : HashMap<BallotChangesKey,BallotsWithGivenTransferValue<Tally>>,
}

impl <Tally:AddAssign> BallotChangesBuilder<Tally> {
    fn to_ballot_changes(self) -> BallotChanges<Tally> {
        let mut changes : Vec<_> = self.map.into_iter().map(|(key,value)|{
            BallotChangeSimilar{
                n: value.n,
                tally: value.tally,
                from: if let Some((tv,candidate)) = key.from { Some(BallotsFromCandidateWithGivenTransferValue{ candidate,ballots:value.ballots,tv})} else {None},
                candidate_to: key.to
            }
        }).collect();
        // do a series of stable sorts to sort by first who from, then who to, then TV.
        changes.sort_by_key(|c|c.from.as_ref().map(|f|f.tv.clone()));
        changes.reverse();
        changes.sort_by_key(|c|c.candidate_to.map(|c|c.0));
        changes.sort_by_key(|c|c.from.as_ref().map(|f|f.candidate.0));
        let n = if changes.is_empty() {BallotPaperCount(0)} else { changes.iter().map(|c|c.n).sum()};
        BallotChanges{ changes,n }
    }
    fn add(&mut self,from:Option<CandidateIndex>, to:Option<CandidateIndex>,found:BallotsWithGivenTransferValue<Tally>) {
        let entry = self.map.entry(BallotChangesKey{from:from.map(|c|(found.tv.clone(),c)),to});
        match &entry {
            Entry::Occupied(_) => {entry.and_modify(|f|f.add(found)); }
            Entry::Vacant(_) => {entry.or_insert(found); }
        }
    }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
/// A "vote level" change - take a number of votes from one candidate and give to another.
pub struct VoteChange<Tally> {
    /// The number of votes to move
    pub n : Tally,
    /// The candidate to move from (or None, if the votes are just to be added)
    pub from : Option<CandidateIndex>,
    /// The candidate to move to (or None, if the votes are just to be added).
    pub to : Option<CandidateIndex>,
}

/// A bunch of votes taken from the same candidate with the same transfer value.
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct BallotsFromCandidateWithGivenTransferValue {
    // the candidate whose votes are being taken away.
    pub candidate : CandidateIndex,
    pub ballots : Vec<TakeVotes>,
    pub tv : TransferValue,
}

/// A concrete set of ballot level changes that are all similar - same TV, same source candidate, same destination candidate.
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct BallotChangeSimilar<Tally> {
    pub n : BallotPaperCount,
    pub tally: Tally,
    pub from : Option<BallotsFromCandidateWithGivenTransferValue>,
    pub candidate_to : Option<CandidateIndex>,
}

/// A concrete set of ballot level changes.
/// They can be applied to vote data by calling [`Self::apply_to_votes`].
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct BallotChanges<Tally> {
    pub changes : Vec<BallotChangeSimilar<Tally>>,
    pub n : BallotPaperCount,
}

impl <Tally> BallotChanges<Tally> {
    pub fn apply_to_votes(&self,election_data:&ElectionData) -> ElectionData {
        let mut data = election_data.clone();
        let num_atl = data.atl.len();
        for change in &self.changes {
            if let Some(from) = change.from.as_ref() {
                for wv in &from.ballots {
                    if wv.from.0<num_atl { // It is an ATL vote
                        data.atl[wv.from.0].n-=wv.n;
                        if let Some(to) = change.candidate_to {
                            let from_party = election_data.metadata.candidate(from.candidate).party.unwrap(); // must have a party or couldn't be in an ATL vote.
                            if let Some(to_party) = election_data.metadata.candidate(to).party {
                                let new_parties = data.atl[wv.from.0].parties.iter().filter(|&&c|c!=to_party).map(|&c|if c==from_party {to_party} else {c}).collect();
                                data.atl.push(ATL{ parties: new_parties, n: wv.n })
                            } else {
                                panic!("Candidate {} got ATL vote but doesn't have a party.",election_data.metadata.candidate(from.candidate).name);
                            }
                        }
                    } else { // It is a BTL vote.
                        data.btl[wv.from.0-num_atl].n-=wv.n;
                        if let Some(to) = change.candidate_to {
                            let new_candidates = data.btl[wv.from.0-num_atl].candidates.iter().filter(|&&c|c!=to).map(|&c|if c==from.candidate {to} else {c}).collect();
                            data.btl.push(BTL{ candidates: new_candidates, n: wv.n })
                        }
                    }

                }
            } else {
                if let Some(to) = change.candidate_to { // insert votes
                    data.btl.push(BTL{ candidates:vec![to], n: change.n.0});
                } else { eprintln!("Trying to do a vote change that does nothing."); } // don't actually do anything...
            }
        }
        data
    }
}
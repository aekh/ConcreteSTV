mod rules;

use clap::{AppSettings, Clap};
use std::path::PathBuf;
use std::fs::File;
use stv::election_data::ElectionData;
use crate::rules::Rules;
use stv::tie_resolution::TieResolutionsMadeByEC;
use std::collections::HashSet;
use stv::ballot_metadata::{NumberOfCandidates, CandidateIndex};
use anyhow::anyhow;
use std::iter::FromIterator;

#[derive(Clap)]
#[clap(version = "0.1", author = "Andrew Conway")]
#[clap(setting = AppSettings::ColoredHelp)]
/// Count STV elections using a variety of rules including good approximations to
/// those used by various electroral commissions on various elections.
struct Opts {
    /// The counting rules to use.
    /// Currently supported AEC2013, AEC2016, AEC2019, Federal
    rules : Rules,

    /// The name of the .stv file to get votes from
    #[clap(parse(from_os_str))]
    votes : PathBuf,

    /// The number of people to elect. If used, overrides the value in the .stv file.
    #[clap(short, long)]
    vacancies : Option<usize>,

    /// An optional .transcript file to store the output in.
    /// If not specified, defaults to votes_rules.transcript where votes and rules are from above.
    #[clap(short, long,parse(from_os_str))]
    transcript : Option<PathBuf>,

    /// An optional list of candidates to exclude.
    #[clap(short, long,use_delimiter=true)]
    exclude : Option<Vec<usize>>,

}


fn main() -> anyhow::Result<()> {
    let opt : Opts = Opts::parse();

    let votes : ElectionData = {
        let file = File::open(&opt.votes)?;
        serde_json::from_reader(file)?
    };

    let vacancies=opt.vacancies.map(|n|NumberOfCandidates(n)).or(votes.metadata.vacancies).ok_or_else(||anyhow!("Need to specify number of vacancies"))?;

    let excluded = match &opt.exclude {
        None => Default::default(),
        Some(v) => HashSet::from_iter(v.iter().map(|c|CandidateIndex(*c))),
    };
    let transcript = opt.rules.count(&votes,vacancies,&excluded,&TieResolutionsMadeByEC::default());

    let transcript_file = match &opt.transcript {
        None => {
            let votename = opt.votes.file_name().map(|o|o.to_string_lossy()).unwrap_or_default();
            let votename = votename.trim_end_matches(".stv");
            let rulename = opt.rules.to_string();
            let combined = votename.to_string()+"_"+&rulename+".transcript";
            opt.votes.with_file_name(combined)
        }
        Some(tf) => tf.clone(),
    };

    if let Some(parent) = transcript_file.parent() { std::fs::create_dir_all(parent)? }
    serde_json::to_writer(File::create(&transcript_file)?,&transcript)?;

    Ok(())
}

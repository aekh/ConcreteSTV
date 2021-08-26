use stv::compare_rules::CompareRules;
use federal::{FederalRulesUsed2013, FederalRulesUsed2016, FederalRulesUsed2019, FederalRules};


fn main()  -> anyhow::Result<()> {

    let loader13 = federal::parse::get_federal_data_loader_2013();
    let loader16 = federal::parse::get_federal_data_loader_2016();
    let loader19 = federal::parse::get_federal_data_loader_2019();
    let iterator = loader13.all_states_data().chain(loader16.all_states_data()).chain(loader19.all_states_data());
    let comparer = CompareRules{ dir: "Comparison/Federal".to_string() };
    // comparer.compute_dataset::<usize,FederalRulesUsed2013,FederalRulesUsed2016,FederalRulesUsed2019,FederalRules>(&data)?;

    comparer.compare_datasets::<usize,FederalRulesUsed2013,FederalRulesUsed2016,FederalRulesUsed2019,FederalRules,_>(iterator)?;


    Ok(())
}
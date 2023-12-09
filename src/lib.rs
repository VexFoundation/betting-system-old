use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, AccountId, Promise, PromiseResult, near_bindgen, require, log, ONE_YOCTO};
use serde_json::json;

const ONE_USDC: f64 = 1000000000000000000000000.0;
const USDC_CONTRACT: &'static str = "cusd.fakes.testnet";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct MatchList {
    future_matches: UnorderedMap<String, Match>, // Map of future matches, match_id is the key
    in_progress_matches: UnorderedMap<String, Match>, // Map of in progress matches
    complete_matches: UnorderedMap<String, Match>, // Map of completed matches
    error_matches: UnorderedMap<String, Match>, // Map of matches that an error has occured e.g. player dropped out
    bet_counter: f64, // Created by summing up all the absolute values of potential_winnings over all games
}

// Struct that holds the details of a match and the bets made in a match
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
struct Match { 
    bets: Vec<Bet>, // List of bets made on a match
    team_1: String,
    team_2: String,
    team_1_total_bets: f64, 
    team_2_total_bets: f64, 
    promised_winnings: f64, 
    winner: Option<String>,
    match_state: MatchState,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
struct Bet {  // Struct that holds the details of a single bet 
    bettor: AccountId,
    decision: String,
    bet_amount: f64,
    potential_winnings: f64, 
    payed_out: PayedOut,
}
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Copy, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum MatchState {
    Future,
    InProgress,
    Complete,
    Error,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Copy, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum PayedOut {
    YetToBePayed,
    Payed,
    ReturnPay,
    NotPayed,
}

// Default implementation for MatchList that creates new maps and bet_counter
impl Default for MatchList {
    fn default() -> Self {
      Self{future_matches: UnorderedMap::new(b"f"), 
        in_progress_matches: UnorderedMap::new(b"p"), 
        complete_matches: UnorderedMap::new(b"c"), 
        error_matches: UnorderedMap::new(b"e"), 
        bet_counter: 0.0}
    }
  }


#[near_bindgen]
impl MatchList { // Implementation of MatchList

    // Call function that allows the user to make a bet on a future match on either team 1 or team 2 in USDC
    pub fn ft_on_transfer(&mut self, sender_id: String, amount: String, msg: String) -> String {
        let bettor: AccountId = sender_id.try_into().unwrap();
        let bet_amount: f64 = amount.parse::<f64>().unwrap();

        // Struct to parse into 
        #[derive(Debug, Deserialize)]
        #[serde(crate = "near_sdk::serde")]
        struct ParsedData {
            match_id: String,
            decision: String,
        }

        // Parse msg from json to varaibles
        let parsed_data: ParsedData = serde_json::from_str(&msg).unwrap_or_else(|err: serde_json::Error| panic!("Invalid json {}", err));
        let match_id: String = parsed_data.match_id;
        let decision: String = parsed_data.decision;

        let bet_amount: f64 = bet_amount / ONE_USDC; // Gets the amount attatched to the bet

        // Finds the relevent match
        let mut current_match: Match = self.future_matches.get(&match_id).unwrap_or_else(|| panic!("No match exists with match)id: {}", match_id));

        require!(current_match.match_state == MatchState::Future, "The game is complete or in progress"); // Match state isn't Future
        
        // Calculates how much will be payed out, will change as odds change with amount betted
        let mut potential_winnings: f64 = 0.0;
        if decision == current_match.team_1 { // If they have picked team 1
            potential_winnings = find_winnings(current_match.team_1_total_bets, current_match.team_2_total_bets, bet_amount);
        } else if decision == current_match.team_2 { // If they have picked team 2
            potential_winnings = find_winnings(current_match.team_2_total_bets, current_match.team_1_total_bets, bet_amount);
        } else { // If not inputted correct team
            panic!("That is not a valid team")
        }

        self.bet_counter -= current_match.promised_winnings.abs(); // Takes off the absolute promised winnings as they will change

        // Adds the bet to the total bets and changes the promised_winnings for that match
        if decision == current_match.team_1 { // If they have picked team 1
            current_match.team_1_total_bets += bet_amount;
            current_match.promised_winnings += potential_winnings;
        } else if decision == current_match.team_2 { // If they have picked team 2
            current_match.team_2_total_bets += bet_amount;
            current_match.promised_winnings -= potential_winnings;
        }
        

        self.bet_counter += current_match.promised_winnings.abs(); // Adds this back on with changed amount
        if self.bet_counter >= (env::account_balance() / ONE_USDC as u128) as f64 { // If the bet counter is larger than amount available in the contract then panics
            panic!("Sorry you can't make a bet as we wouldn't definetly be able to pay out")
        }

        let payed_out: PayedOut = PayedOut::YetToBePayed; 
        // Potential winnings are stored in yoctoNEAR
        let new_bet: Bet = Bet{bettor, decision: decision.clone(), bet_amount: bet_amount.clone(), potential_winnings: potential_winnings.clone(), payed_out: payed_out.clone()}; // Creates a new bet with the fields filled in
        current_match.bets.push(new_bet); // Pushes the new bet to the bets list for that match
        self.future_matches.insert(&match_id, &current_match); // Updates the match
        log!("You have made a bet on {}, with ${} , at odds {}, and potential winnings {}", decision, bet_amount, potential_winnings / bet_amount, potential_winnings);

        return "0".to_string()
    }

    // View function that allows the user to view all future matches
    pub fn view_future_matches(&self, match_id: String) -> Vec<(String, String, f64, String, f64, Option<String>, MatchState)> {
        let mut match_list: Vec<(String, String, f64, String, f64, Option<String>, MatchState)> = Vec::new(); // Creates a new empty list where the required values will get added to 

        if match_id == "all" {
            let keys: Vec<String> = (self.future_matches.keys_as_vector()).to_vec(); // Converts all the matches keys into a vector 
            let values: Vec<Match> = (self.future_matches.values_as_vector()).to_vec(); // Converts all matches values into a vector
            for i in 0..self.future_matches.len() { // Loops the length of the list
                let i: usize = i.try_into().unwrap(); // Converts i to usize to index properly
                let key: String = keys[i].to_string(); // Converts values to a form that can be used
                let team_name_1: String = (values[i].team_1).to_string();
                let team_name_2: String = (values[i].team_2).to_string();
                let winner: Option<String> = values[i].winner.clone();
                let match_state: MatchState = values[i].match_state;
                let odds: (f64, f64) = find_starting_odds(values[i].team_1_total_bets, values[i].team_2_total_bets); // Gets the odds for the game
                let individual_match: (String, String, f64, String, f64, Option<String>, MatchState) = (key, team_name_1, odds.0, team_name_2, odds.1, winner, match_state); // Creates a tuple of infomation
                match_list.push(individual_match) // Pushes this tuple to the list
            }
        } else {
            let current_match: Match = self.future_matches.get(&match_id).unwrap_or_else(|| panic!("No match exists with match)id: {}", match_id)); // Finds the desired match, panics if doesn't find the match
            let team_name_1: String = (current_match.team_1).to_string();
            let team_name_2: String = (current_match.team_2).to_string();
            let winner: Option<String> = current_match.winner;
            let match_state: MatchState = current_match.match_state;
            let odds: (f64, f64) = find_starting_odds(current_match.team_1_total_bets, current_match.team_2_total_bets); // Gets the odds for the game
            let individual_match: (String, String, f64, String, f64, Option<String>, MatchState) = (match_id, team_name_1, odds.0, team_name_2, odds.1, winner, match_state); // Creates a tuple of infomation
            match_list.push(individual_match) // Pushes this tuple to the list
        }

        match_list // Returns the list
    }

    // Private call function that allows the contract account to create a new match, need to input teams, odds and the date of the match
    #[private]
    pub fn create_match(&mut self, team_1: String, team_2: String, in_odds_1: f64, in_odds_2: f64, date: String) {
        let match_id: String = format!("{}-{}-{}", team_1, team_2, date); // The match_id is formed from the team names and the date

        // Creates inital bets pool inline with odds
        let in_prob_1: f64 = 1.0 / in_odds_1; // Changes initial decimal odds to initial probability
        let in_prob_2: f64 = 1.0 / in_odds_2;
        let divider: f64 = in_prob_1 + in_prob_2; // Creates the divider by adding implied odds
        let actual_prob_1: f64 = in_prob_1 / divider; // Divides initial probability to give actual probability
        let actual_prob_2: f64 = in_prob_2 / divider;
        let team_1_total_bets: f64 = (actual_prob_1 * 1000.0).round(); // Sets the initial bets and rounds, multiplies by weighting of 1000
        let team_2_total_bets: f64 = (actual_prob_2 * 1000.0).round();

        let bets: Vec<Bet> = Vec::new(); // Creates a new empty bets list that holds all the bets
        let winner: Option<String> = None;
        let match_state: MatchState = MatchState::Future;
        let promised_winnings: f64 = 0.0;
        let new_match: Match = Match{bets, team_1, team_2, team_1_total_bets, team_2_total_bets, promised_winnings, winner, match_state}; // Creates a new_match using the Match struct
        self.future_matches.insert(&match_id, &new_match); // Adds this new_match to the future_matches map
        log!("A new match has been added with ID {}", match_id)
    }


    // Private call function that moves a match from future to in progress, done at the start of the match, can no longer bet
    #[private]
    pub fn end_betting(&mut self, match_id: String) {
        let mut current_match: Match = self.future_matches.get(&match_id).unwrap_or_else(|| panic!("No match exists with match)id: {}", match_id)); // Finds the desired match, panics if doesn't find the match
        
        require!(current_match.match_state == MatchState::Future, "That game is already complete or in progress"); // Checks that the game has not already been ended

        current_match.match_state = MatchState::InProgress;
        self.bet_counter -= current_match.promised_winnings.abs(); // Removes the promised winnings from the bet_counter
        self.in_progress_matches.insert(&match_id, &current_match); // Inserts the match into in_progress_matches
        self.future_matches.remove(&match_id); // Removes the match from future_matches
    }


    // Private call function that allows the contract account to finish a match, need to input the winning team
    #[private]
    pub fn finish_match(&mut self, match_id: String, winning_team: String) {
        let mut current_match: Match = self.in_progress_matches.get(&match_id).unwrap_or_else(|| panic!("No match exists with match)id: {}", match_id)); // Finds the desired match, panics if doesn't find the match
        
        require!(current_match.match_state == MatchState::InProgress, "That game is already complete or in the future"); // Checks that the game has not already been ended
        require!(winning_team == current_match.team_1 || winning_team == current_match.team_2, "That is not a valid team"); // Checks valid winner input 

        for i in 0..current_match.bets.len() { // Loops through all bets
            if current_match.bets[i].payed_out == PayedOut::YetToBePayed { // Checks not already payed out and they bet on the winner
                if current_match.bets[i].decision == winning_team { // Checks they bet on the winner
                    
                    let winner: AccountId = current_match.bets[i].bettor.clone(); // Gets the account Id of each winner
                    let winnings: f64 = current_match.bets[i].potential_winnings * ONE_USDC; // Gets the amount they win
                    let args: Vec<u8> = json!({
                        "receiver_id": winner,
                        "amount": winnings.to_string(),
                        "memo": "Winnings",
                    }).to_string().into_bytes();
                    Promise::new(USDC_CONTRACT.parse().unwrap()).function_call("ft_transfer".to_string(), args, ONE_YOCTO, near_sdk::Gas(100000000000000));
                    //Change to ft_transfer_call

                    current_match.bets[i].payed_out = PayedOut::Payed;
                }
                
            }
        } 

        current_match.winner = Some(winning_team); // Sets the winning team
        current_match.match_state = MatchState::Complete;
        self.complete_matches.insert(&match_id, &current_match); // Inserts the match into the complete_matches
        self.in_progress_matches.remove(&match_id); // Removes the match from in_progress_matches
        log!("The match is now complete")
    }


    // Private call functio nthat allows the contract account to return funds to the bettors if a match was cancelled
    #[private]
    pub fn return_funds(&mut self, match_id: String, state: MatchState) {
        let mut current_match: Option<Match> = None;

        match state {
            MatchState::Future => {
                current_match = Some(self.future_matches.get(&match_id).unwrap_or_else(|| panic!("No match exists with match)id: {}", match_id))); // Finds the desired match, panics if doesn't find the match
            }
            MatchState::InProgress => {
                current_match = Some(self.in_progress_matches.get(&match_id).unwrap_or_else(|| panic!("No match exists with match)id: {}", match_id))); // Finds the desired match, panics if doesn't find the match
            }
            _ => panic!("That is not a valid state"),
        }

        match current_match { Some(mut x) => { // If there is a match
            for i in 0..x.bets.len() { // Loops through all bets
                if x.bets[i].payed_out == PayedOut::YetToBePayed { // Checks not already payed out and they bet on the winner
                    // Payout this person (convert to balance)
                    let account: AccountId = x.bets[i].bettor.clone();
                    let returns: f64 = x.bets[i].bet_amount * ONE_USDC;

                    let args: Vec<u8> = json!({
                        "receiver_id": account,
                        "amount": returns.to_string(),
                        "memo": "Return funds",
                    }).to_string().into_bytes();
                    Promise::new(USDC_CONTRACT.parse().unwrap()).function_call("ft_transfer".to_string(), args, ONE_YOCTO, near_sdk::Gas(30000000000000));

                    //Extra checks
                    //Update bet payed out for each individual sequencially not at end as one might be payed out but not others
                    x.bets[i].payed_out = PayedOut::ReturnPay;       
                }
            } 

            log!("Return pay has been issued");
        
            x.match_state = MatchState::Error;
            self.error_matches.insert(&match_id, &x); // Inserts the match into the complete_matches

            match state {
                MatchState::Future => {
                    self.bet_counter -= x.promised_winnings.abs(); // Removes the promised winnings from the bet_counter
                    self.future_matches.remove(&match_id); // Removes the match from future_matches
                }
                MatchState::InProgress => {
                    self.in_progress_matches.remove(&match_id); // Removes the match from in_progress_matches
                }
                _ => panic!("That is not a valid state")
            }

        } None => { 
            panic!("Error")
        } } 
    }


    // View function that allows the user to view the bets for a single match
    // Input either the bet ID to view a single bet or "all" to view all bets for that match
    pub fn view_bets(&self, match_id: String, name: String) -> Vec<(String, String, f64, f64, PayedOut)> {
        let current_match: Match = self.future_matches.get(&match_id).unwrap_or_else(|| panic!("No match exists with match)id: {}", match_id));  // Finds the desired match, panics if doesn't find the match
        let mut bet_list: Vec<(String, String, f64, f64, PayedOut)> = Vec::new(); // Creates a new empty list where the required values will get added to 
        for i in 0..current_match.bets.len() { // Loops through all the bets for the match
            let i: usize = i.try_into().unwrap();
            let username: String = (current_match.bets[i].bettor).to_string();
            if name == "all" || name == username { // If all selected then it will selected all the bets, if not it will selected the bets with the correct name
                let team: String = (current_match.bets[i].decision).to_string(); // Seperates the infomation from the Bet struct into variables
                let bet: f64 = current_match.bets[i].bet_amount;
                let winnings: f64 = current_match.bets[i].potential_winnings;
                let payed: PayedOut = current_match.bets[i].payed_out;

                let individual_bet: (String, String, f64, f64, PayedOut) = (username, team, bet, winnings, payed); // Creates a tuple containing the information
                bet_list.push(individual_bet); } // Adds the tuple to the list of bets that are to be displayed
            }

        bet_list // Returns the list of bets

    }


    // View function that allows the user to view what the potential winnings would be if they placed a bet on a certain match, on a certain team, with a certain amount
    pub fn view_potential_winnings(&self, match_id: String, team: String, bet_amount: String) -> f64 {
        let current_match: Match = self.future_matches.get(&match_id).unwrap_or_else(|| panic!("No match exists with match)id: {}", match_id)); // Finds the desired match, panics if doesn't find the match

        if team != current_match.team_1 && team != current_match.team_2 { // Checks valid team input 
            panic!("That is not a valid team")
        }

        let bet_amount_f64: f64 = bet_amount.parse().unwrap();

        let mut potential_winnings: f64 = 0.0;
        if team == current_match.team_1 { // If they have picked team 1
            potential_winnings = find_winnings(current_match.team_1_total_bets, current_match.team_2_total_bets, bet_amount_f64);
        } else if team == current_match.team_2 { // If they have picked team 2
            potential_winnings = find_winnings(current_match.team_2_total_bets, current_match.team_1_total_bets, bet_amount_f64);
        }

        potential_winnings // Displays the potential winnings

    }


}


// Function that can only be called by the code. Gets the starting odds from the total bets and adds to 5% take
fn find_starting_odds(team_1_total_bets: f64, team_2_total_bets: f64) -> (f64, f64) {
    let total: f64 = team_1_total_bets + team_2_total_bets;
    let divider: f64 = total / 1.05; // Gives the divider that makes implied probability add to 1.05
    let implied_prob_1: f64 = team_1_total_bets / divider; // Finds the implied probabilty
    let implied_prob_2: f64 = team_2_total_bets / divider;
    let team_1_odds: f64 = (100.0 / implied_prob_1).round() / 100.0; // Gives the odds 
    let team_2_odds: f64 = (100.0 / implied_prob_2).round() / 100.0;

    (team_1_odds, team_2_odds)
}


// Function that can only be called by the code. Finds the potentail winnings for a bet
// The team that is being betted on goes first in the function call, and the other team is second
// Intergrates over odds with bet amount
fn find_winnings(betted_team_bets: f64, other_team: f64, bet_amount: f64) -> f64 {
    let ln_target: f64 = (betted_team_bets + bet_amount) / betted_team_bets;
    (1.0 / 1.05) * (bet_amount + other_team * ln_target.ln())
}


use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, AccountId, Balance, Promise, PromiseResult, near_bindgen, log, ONE_NEAR};


#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
struct Bet {  // Struct that holds the details of a single bet 
    bettor: AccountId,
    decision: String,
    bet_amount: f64, // Stored in NEAR //Has to be transfered to balance when paying by converting multiplying by ONE_NEAR
    potential_winnings: f64, // Stored in NEAR
    payed_out: bool,
}

// Struct that holds the details of a match and the bets made in a match
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
struct Match { 
    bets: Vec<Bet>,
    team_1: String,
    team_2: String,
    team_1_total_bets: f64, // Stored in NEAR
    team_2_total_bets: f64, // Stored in NEAR
    promised_winnings: f64, // Stores the maximum payout amount, team 1 add to this, team 2 take away from this
    winner: Option<String>,
    finished: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct MatchList { // Main struct that holds all the matches
  matches: UnorderedMap<String, Match>, // Key is match ID
  bet_counter: f64, // Created by summing up all the absolute values of potential_winnings over all games
}

// Default implementation for MatchList that creates a new matches UnorderedMap
impl Default for MatchList {
    fn default() -> Self {
      Self{matches: UnorderedMap::new(b"m"), bet_counter: 0.0}
    }
  }


#[near_bindgen]
impl MatchList { // Implementation of MatchList

    // Call function that allows the user to make a bet on a match either team 1 or team 2 and attatch NEAR 
    #[payable]
    pub fn make_bet(&mut self, match_id: String, decision: String) {
        let bettor: AccountId = env::signer_account_id(); // Gets the signers account ID
        let bet_amount = env::attached_deposit() as f64 / ONE_NEAR as f64; // Gets the amount of near attatched to the bet#
        assert!(bet_amount >= 0.1, "You need to attatch atleast 0.1 NEAR"); // Makes sure NEAR is attatched
        // Add function that checks if the bet is too large as in the system is able to pay out

        // Finds the match we are talking about
        let mut current_match = self.matches.get(&match_id).expect("No match exists with that id"); // Panics if not found

        assert!(current_match.finished == false, "The game is finished");
        
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

        // Adds the bet to the total bets for that match
        if decision == current_match.team_1 { // If they have picked team 1
            current_match.team_1_total_bets += bet_amount;
            current_match.promised_winnings += potential_winnings;
        } else if decision == current_match.team_2 { // If they have picked team 2
            current_match.team_2_total_bets += bet_amount;
            current_match.promised_winnings -= potential_winnings;
        }
        

        self.bet_counter += current_match.promised_winnings.abs(); // Adds this back on with changed amount

        if self.bet_counter >= (env::account_balance() / ONE_NEAR) as f64 { // If the bet counter is larger than amount available in the contract then panics (includes attatched NEAR)
            panic!("Sorry you can't make a bet as we wouldn't definetly be able to pay out")
        }

        let payed_out = false;
        // Potential winnings are stored in yoctoNEAR
        let new_bet = Bet{bettor, decision: decision.clone(), bet_amount: bet_amount.clone(), potential_winnings: potential_winnings.clone(), payed_out: payed_out.clone()}; // Creates a new bet with the fields filled in
        current_match.bets.push(new_bet); // Pushes the new bet to the bets list for that match
        self.matches.insert(&match_id, &current_match); // Updates the match
        log!("You have made a bet on {}, with {} NEAR, at odds {}, and potential winnings {}", decision, bet_amount, potential_winnings / bet_amount, potential_winnings)

    }

    // View function that allows the user to view all not finished matches with the odds
    pub fn view_matches(&self, match_id: String) -> Vec<(String, String, f64, String, f64, Option<String>, bool)> {
        let mut match_list = Vec::new(); // Creates a new empty list where the required values will get added to 

        if match_id == "all" {
            let keys = (self.matches.keys_as_vector()).to_vec(); // Converts all the matches keys into a vector 
            let values = (self.matches.values_as_vector()).to_vec(); // Converts all matches values into a vector
            for i in 0..self.matches.len() { // Loops the length of the list
                let i: usize = i.try_into().unwrap(); // Converts i to usize to index properly
                let key = keys[i].to_string(); // Converts values to a form that can be used
                let team_name_1 = (values[i].team_1).to_string();
                let team_name_2 = (values[i].team_2).to_string();
                let winner = values[i].winner.clone();
                let finished = values[i].finished;
                log!("team 1 pool {} team 2 pool {} promised winnings {}", values[i].team_1_total_bets, values[i].team_2_total_bets, values[i].promised_winnings); // REMOVE
                let odds = find_starting_odds(values[i].team_1_total_bets, values[i].team_2_total_bets); // Gets the odds for the game
                let individual_match = (key, team_name_1, odds.0, team_name_2, odds.1, winner, finished); // Creates a tuple of infomation
                match_list.push(individual_match) // Pushes this tuple to the list
            }
        } else {
            let current_match = self.matches.get(&match_id).expect("No match exists with that id"); // Panics if doesn't find the match
            let team_name_1 = (current_match.team_1).to_string();
            let team_name_2 = (current_match.team_2).to_string();
            let winner = current_match.winner;
            let finished = current_match.finished;
            let odds = find_starting_odds(current_match.team_1_total_bets, current_match.team_2_total_bets); // Gets the odds for the game
            let individual_match = (match_id, team_name_1, odds.0, team_name_2, odds.1, winner, finished); // Creates a tuple of infomation
            match_list.push(individual_match) // Pushes this tuple to the list
        }


        log!(" bet counter {}", self.bet_counter); //REMOVE
        match_list // Returns the list
    }

    // Private call function that allows the contract account to create a new match, input odds and teams
    #[private]
    pub fn create_match(&mut self, team_1: String, team_2: String, in_odds_1: String, in_odds_2: String, date: String) {
        let in_odds_1 = in_odds_1.parse::<f64>().unwrap(); // Convert to f64
        let in_odds_2 = in_odds_2.parse::<f64>().unwrap();
        let match_id: String = format!("{}-{}-{}", team_1, team_2, date); // The match_id is formed from the team names and the date

        // Create initial fake bets
        let in_prob_1 = 1.0 / in_odds_1; // Changes initial decimal odds to initial probability
        let in_prob_2 = 1.0 / in_odds_2;
        let divider = in_prob_1 + in_prob_2; // Creates the divider by adding implied odds
        let actual_prob_1 = in_prob_1 / divider; // Divides initial probability to give actual probability
        let actual_prob_2 = in_prob_2 / divider;
        let team_1_total_bets = (actual_prob_1 * 1000.0).round(); // Sets the initial bets and rounds, multiplies by weighting of 1000
        let team_2_total_bets = (actual_prob_2 * 1000.0).round();

        let bets = Vec::new(); // Creates a new empty bets list that holds all the bets
        let winner = None;
        let finished = false;
        let promised_winnings = 0.0;
        let new_match = Match{bets, team_1, team_2, team_1_total_bets, team_2_total_bets, promised_winnings, winner, finished}; // Creates a new_match using the Match struct
        self.matches.insert(&match_id, &new_match); // Adds this new_match to the matches list in the MatchList struct
        log!("A new match has been added with ID {}", match_id)
    }

    // Private call function that allows the contract account to finish a match, input the winning team
    #[private]
    #[payable]
    pub fn finish_match(&mut self, match_id: String, winning_team: String) {
        let mut current_match = self.matches.get(&match_id).expect("No match exists with that id"); // Panics if doesn't find the match
        
        if current_match.finished == true { // Checks that the game has not already been ended
            panic!("That game has already ended")
        }

        if winning_team != current_match.team_1 && winning_team != current_match.team_2 { // Checks valid winner input 
            panic!("That is not a valid team")
        }


        for i in 0..current_match.bets.len() { // Loops through all bets
            if current_match.bets[i].payed_out == false { // Checks not already payed out and they bet on the winner
                if current_match.bets[i].decision == winning_team { // Checks they bet on the winner
                    // Payout this person (convert to balance)
                    let winner: AccountId = current_match.bets[i].bettor.clone();
                    let winnings: f64 = current_match.bets[i].potential_winnings;
                    Promise::new(winner.clone()) // Promise to the account that made the bet
                        .transfer(( winnings * (ONE_NEAR as f64)) as u128 ); // Transfers the potential winnings converted to Balance type (u128) in yoctoNEAR
                    log!("Transfered {} NEAR to {}", winnings, winner); 
                    // Need to add in an error checking function here
                    current_match.bets[i].payed_out = true;
                }
                
            }
        } 

        current_match.winner = Some(winning_team);
        current_match.finished = true;
        self.bet_counter -= current_match.promised_winnings.abs(); // Removes the promised winnings from the bet_counter
        self.matches.insert(&match_id, &current_match); // Updates the match

    }

    // View function that allows the contract account to view the bets for a single match
    // Input either the bet ID to view a single bet or "all" to view all bets for that match
    pub fn view_bets(&self, match_id: String, name: String) -> Vec<Bet>{
        let current_match = self.matches.get(&match_id).expect("No match exists with that id"); // Panics if doesn't find the match
        if name == "all" {
            current_match.bets.into_iter().collect()
        } else {
            current_match.bets.into_iter().filter(|bet| bet.bettor.to_string() == name).collect()
        }

    }

}

// Function that can only be called by the code. Gets the starting odds from the total bets and adds to 5% take
fn find_starting_odds(team_1_total_bets: f64, team_2_total_bets: f64) -> (f64, f64) {
    let total = team_1_total_bets + team_2_total_bets;
    let divider = total / 1.05; // Gives the divider that makes implied probability add to 1.05
    let implied_prob_1 = team_1_total_bets / divider; // Finds the implied probabilty
    let implied_prob_2 = team_2_total_bets / divider;
    let team_1_odds = (100.0 / implied_prob_1).round() / 100.0; // Gives the odds 
    let team_2_odds = (100.0 / implied_prob_2).round() / 100.0;

    (team_1_odds, team_2_odds)
    }

// Function that can only be called by the code. Finds the potentail winnings for a bet
// The team that is being betted on goes first in the function call, and the other team is second
// Intergrates over odds with bet amount
fn find_winnings(betted_team_bets: f64, other_team: f64, bet_amount: f64) -> f64 {
    let ln_target: f64 = (betted_team_bets + bet_amount) / betted_team_bets;
    (1.0 / 1.05) * (bet_amount + other_team * ln_target.ln())
}
